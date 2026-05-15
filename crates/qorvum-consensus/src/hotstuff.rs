//! HotStuff BFT Core Structures

use qorvum_crypto::signing::{Signature, PublicKey};
use serde::{Deserialize, Serialize};

/// Represents a vote from a validator on a specific block hash in a specific view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoteMessage {
    pub view_number: u64,
    pub block_hash: [u8; 32],
    pub validator_pub_key: PublicKey,
    pub signature: Signature,
}

/// A Quorum Certificate (QC) is proof that 2f+1 validators voted for a block in a view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuorumCertificate {
    pub view_number: u64,
    pub block_hash: [u8; 32],
    pub signatures: Vec<(PublicKey, Signature)>,
}

impl QuorumCertificate {
    pub fn genesis() -> Self {
        Self { view_number: 0, block_hash: [0u8; 32], signatures: vec![] }
    }
}

/// A Proposal message sent by the Leader of the current view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposalMessage {
    pub view_number: u64,
    /// Serialized (Block, Vec<RecordOp>) payload
    pub block_data: Vec<u8>,
    pub justify_qc: QuorumCertificate,
    pub leader_signature: Signature,
}

/// Top-level message type for the qorvum-consensus gossipsub topic.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsensusMsg {
    Proposal(ProposalMessage),
    Vote(VoteMessage),
    Qc(QuorumCertificate),
}

/// Hash block_data to derive the canonical block hash used for voting.
pub fn hash_block_data(block_data: &[u8]) -> [u8; 32] {
    qorvum_crypto::hash(block_data)
}

/// The Validator Set manages who is allowed to participate in consensus.
pub struct ValidatorSet {
    pub authorized_keys: Vec<PublicKey>,
}

impl ValidatorSet {
    pub fn new(keys: Vec<PublicKey>) -> Self {
        Self { authorized_keys: keys }
    }

    pub fn is_authorized(&self, key: &PublicKey) -> bool {
        self.authorized_keys.iter().any(|k| k.bytes == key.bytes)
    }

    /// Required quorum size: 2f + 1, where f = (N - 1) / 3
    pub fn quorum_size(&self) -> usize {
        let n = self.authorized_keys.len();
        if n == 0 { return 0; }
        let f = (n - 1) / 3;
        (2 * f) + 1
    }

    pub fn verify_vote(&self, vote: &VoteMessage) -> bool {
        if !self.is_authorized(&vote.validator_pub_key) {
            tracing::warn!("Vote rejected: unregistered validator");
            return false;
        }
        let mut payload = vote.view_number.to_be_bytes().to_vec();
        payload.extend_from_slice(&vote.block_hash);
        qorvum_crypto::signing::verify(&vote.validator_pub_key, &payload, &vote.signature)
    }

    pub fn verify_qc(&self, qc: &QuorumCertificate) -> bool {
        if qc.view_number == 0 { return true; } // genesis QC always valid
        let mut payload = qc.view_number.to_be_bytes().to_vec();
        payload.extend_from_slice(&qc.block_hash);

        let mut valid_voters = std::collections::HashSet::new();
        for (pub_key, sig) in &qc.signatures {
            if !self.is_authorized(pub_key) { continue; }
            if qorvum_crypto::signing::verify(pub_key, &payload, sig) {
                valid_voters.insert(pub_key.to_hex());
            }
        }

        let required = self.quorum_size();
        let achieved  = valid_voters.len();
        if achieved >= required {
            tracing::info!("QC verified: {}/{} votes", achieved, required);
            true
        } else {
            tracing::warn!("QC failed: {}/{} valid votes", achieved, required);
            false
        }
    }
}
