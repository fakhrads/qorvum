//! HotStuff BFT Consensus Engine
//!
//! Flow (leader path):
//!   propose_block(block_data) → broadcast Proposal → self-vote → collect Votes
//!   → 2f+1 reached → form QC → commit block → broadcast QC → resolve promise
//!
//! Flow (validator path):
//!   receive Proposal → store block_data → cast Vote via gossipsub
//!   receive QC → commit block

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, oneshot, Mutex};
use tracing::{error, info, warn};

use crate::hotstuff::{
    hash_block_data, ConsensusMsg, ProposalMessage, QuorumCertificate, ValidatorSet, VoteMessage,
};
use qorvum_crypto::signing::{PQKeypair, PublicKey, Signature};
use qorvum_ledger::{
    block::Block,
    store::{LedgerStore, RecordOp},
};

// ── Internal state ────────────────────────────────────────────────────────────

type CommitTx = oneshot::Sender<anyhow::Result<QuorumCertificate>>;

struct Pending {
    #[allow(dead_code)]
    block_hash: [u8; 32],
    block_data: Vec<u8>,
    /// Present only on the node that called propose_block() (the leader).
    commit_tx: Option<CommitTx>,
}

struct EngineState {
    view: u64,
    pending: HashMap<u64, Pending>,
    vote_sigs: HashMap<u64, Vec<(PublicKey, Signature)>>,
    vote_seen: HashMap<u64, HashSet<String>>,
    committed: HashSet<u64>,
}

// ── Public API ────────────────────────────────────────────────────────────────

pub struct ConsensusEngine {
    val_set: ValidatorSet,
    keypair: PQKeypair,
    store: Arc<dyn LedgerStore>,
    /// Channel to the NetworkService consensus gossipsub topic.
    net_tx: mpsc::Sender<Vec<u8>>,
    state: Mutex<EngineState>,
}

impl ConsensusEngine {
    pub fn new(
        val_set: ValidatorSet,
        keypair: PQKeypair,
        store: Arc<dyn LedgerStore>,
        net_tx: mpsc::Sender<Vec<u8>>,
    ) -> Arc<Self> {
        Arc::new(Self {
            val_set,
            keypair,
            store,
            net_tx,
            state: Mutex::new(EngineState {
                view: 0,
                pending: HashMap::new(),
                vote_sigs: HashMap::new(),
                vote_seen: HashMap::new(),
                committed: HashSet::new(),
            }),
        })
    }

    /// Submit a block for consensus. Returns a QC once 2f+1 validators have voted.
    /// The block is committed to the ledger store before this function returns.
    ///
    /// `block_data` must be `bincode::serialize(&(block, ops))`.
    pub async fn propose_block(&self, block_data: Vec<u8>) -> anyhow::Result<QuorumCertificate> {
        let block_hash = hash_block_data(&block_data);
        let (commit_tx, commit_rx) = oneshot::channel();

        let view = {
            let mut s = self.state.lock().await;
            s.view += 1;
            let v = s.view;
            s.pending.insert(
                v,
                Pending { block_hash, block_data: block_data.clone(), commit_tx: Some(commit_tx) },
            );
            v
        };

        // Sign and broadcast proposal
        let leader_sig = self.keypair.sign(&Self::vote_payload(view, block_hash))?;
        let proposal = ProposalMessage {
            view_number: view,
            block_data,
            justify_qc: QuorumCertificate::genesis(),
            leader_signature: leader_sig,
        };
        self.broadcast(ConsensusMsg::Proposal(proposal)).await;

        // Count our own vote (gossipsub does not echo to publisher)
        self.do_vote(view, block_hash).await;

        // Wait up to 30s for quorum
        tokio::time::timeout(Duration::from_secs(30), commit_rx)
            .await
            .map_err(|_| {
                anyhow::anyhow!(
                    "Consensus timeout after 30s — not enough validators online (need {})",
                    self.val_set.quorum_size()
                )
            })?
            .map_err(|_| anyhow::anyhow!("ConsensusEngine dropped before QC was formed"))?
    }

    /// Called by the network task for every message received on the consensus topic.
    pub async fn handle_network_msg(&self, raw: Vec<u8>) {
        match bincode::deserialize::<ConsensusMsg>(&raw) {
            Ok(ConsensusMsg::Proposal(p)) => self.on_proposal(p).await,
            Ok(ConsensusMsg::Vote(v)) => self.on_vote(v).await,
            Ok(ConsensusMsg::Qc(qc)) => self.on_qc(qc).await,
            Err(e) => warn!("Malformed consensus msg: {e}"),
        }
    }
}

// ── Internal helpers ──────────────────────────────────────────────────────────

impl ConsensusEngine {
    fn vote_payload(view: u64, block_hash: [u8; 32]) -> Vec<u8> {
        let mut p = view.to_be_bytes().to_vec();
        p.extend_from_slice(&block_hash);
        p
    }

    async fn broadcast(&self, msg: ConsensusMsg) {
        match bincode::serialize(&msg) {
            Ok(enc) => {
                if let Err(e) = self.net_tx.send(enc).await {
                    error!("Consensus net_tx send failed: {e}");
                }
            }
            Err(e) => error!("Consensus msg serialize failed: {e}"),
        }
    }

    /// Sign and broadcast our vote, then count it locally.
    async fn do_vote(&self, view: u64, block_hash: [u8; 32]) {
        let payload = Self::vote_payload(view, block_hash);
        let sig = match self.keypair.sign(&payload) {
            Ok(s) => s,
            Err(e) => { error!("Failed to sign vote: {e}"); return; }
        };
        let pk = self.keypair.public_key().clone();

        self.broadcast(ConsensusMsg::Vote(VoteMessage {
            view_number: view,
            block_hash,
            validator_pub_key: pk.clone(),
            signature: sig.clone(),
        })).await;

        self.tally_vote(view, block_hash, pk, sig).await;
    }

    /// Add a vote to the accumulator. Triggers finalization when quorum reached.
    async fn tally_vote(
        &self,
        view: u64,
        block_hash: [u8; 32],
        pk: PublicKey,
        sig: Signature,
    ) {
        let quorum = self.val_set.quorum_size();
        let should_finalize = {
            let mut s = self.state.lock().await;
            if s.committed.contains(&view) { return; }

            let voter_hex = pk.to_hex();
            if !s.vote_seen.entry(view).or_default().insert(voter_hex) { return; } // duplicate

            s.vote_sigs.entry(view).or_default().push((pk, sig));
            s.vote_sigs[&view].len() >= quorum
        };

        if should_finalize {
            self.finalize(view, block_hash).await;
        }
    }

    /// Form QC, commit block to store, broadcast QC, resolve pending promise.
    async fn finalize(&self, view: u64, block_hash: [u8; 32]) {
        // Atomically mark committed and extract everything we need
        let (block_data, commit_tx, sigs) = {
            let mut s = self.state.lock().await;
            if s.committed.contains(&view) { return; }
            s.committed.insert(view);

            let pending = match s.pending.remove(&view) {
                Some(p) => p,
                None => {
                    error!("finalize: no pending block for view {view}");
                    return;
                }
            };
            let sigs = s.vote_sigs.remove(&view).unwrap_or_default();
            s.vote_seen.remove(&view);
            (pending.block_data, pending.commit_tx, sigs)
        }; // Mutex released here

        let qc = QuorumCertificate { view_number: view, block_hash, signatures: sigs };

        info!("Quorum reached for view {view} — committing block");

        // Deserialize and commit
        match serde_json::from_slice::<(Block, Vec<RecordOp>)>(&block_data) {
            Ok((block, ops)) => {
                match self.store.commit_block(&block, ops) {
                    Ok(()) => info!("Block {} committed (view {view})", block.header.block_number),
                    Err(e) => {
                        error!("commit_block failed: {e}");
                        if let Some(tx) = commit_tx {
                            let _ = tx.send(Err(anyhow::anyhow!("commit_block: {e}")));
                        }
                        return;
                    }
                }
            }
            Err(e) => {
                error!("Deserialize block_data failed: {e}");
                if let Some(tx) = commit_tx {
                    let _ = tx.send(Err(anyhow::anyhow!("deserialize: {e}")));
                }
                return;
            }
        }

        // Broadcast QC so non-leader validators can commit
        self.broadcast(ConsensusMsg::Qc(qc.clone())).await;

        // Resolve the HTTP handler future
        if let Some(tx) = commit_tx {
            let _ = tx.send(Ok(qc));
        }
    }

    // ── Message handlers ──────────────────────────────────────────────────────

    async fn on_proposal(&self, p: ProposalMessage) {
        let view = p.view_number;
        let block_hash = hash_block_data(&p.block_data);

        // Store block_data so we can commit it when QC arrives (or when we tally enough votes)
        {
            let mut s = self.state.lock().await;
            // or_insert: leader may have already stored it; don't overwrite with our copy
            s.pending.entry(view).or_insert(Pending {
                block_hash,
                block_data: p.block_data,
                commit_tx: None,
            });
        }

        self.do_vote(view, block_hash).await;
    }

    async fn on_vote(&self, v: VoteMessage) {
        if !self.val_set.verify_vote(&v) {
            warn!("Rejected invalid vote for view {}", v.view_number);
            return;
        }
        self.tally_vote(v.view_number, v.block_hash, v.validator_pub_key, v.signature).await;
    }

    async fn on_qc(&self, qc: QuorumCertificate) {
        // Non-leader path: received a valid QC → commit if not already done
        if self.val_set.verify_qc(&qc) {
            self.finalize(qc.view_number, qc.block_hash).await;
        } else {
            warn!("Received invalid QC for view {}", qc.view_number);
        }
    }
}
