//! ConsensusRole — drives HotStuff BFT for all TxSubmissions arriving from NodeBus.

use std::sync::Arc;
use tracing::{error, info, warn};
use tokio::sync::RwLock;

use qorvum_contracts::executor::ContractExecutor;
use qorvum_consensus::ConsensusEngine;
use qorvum_ledger::{
    block::{BlockBuilder, EndorsementSig, Transaction},
    store::LedgerStore,
};

use crate::bus::{BlockCommittedEvent, NodeBus};

pub struct ConsensusRole {
    engine: Arc<ConsensusEngine>,
    store:  Arc<dyn LedgerStore>,
    bus:    NodeBus,
}

impl ConsensusRole {
    pub fn new(
        engine: Arc<ConsensusEngine>,
        store:  Arc<dyn LedgerStore>,
        bus:    NodeBus,
    ) -> Self {
        Self { engine, store, bus }
    }

    pub async fn run(self) {
        info!("[consensus] Role started");

        // Register contracts
        let mut executor = ContractExecutor::new(self.store.clone());
        executor.register_native("hr-service", hr_service::register());
        let executor = Arc::new(RwLock::new(executor));

        // Task: forward inbound P2P consensus messages → ConsensusEngine
        let engine_for_p2p = self.engine.clone();
        let p2p_in_rx = self.bus.p2p_in_receiver();
        tokio::spawn(async move {
            loop {
                let msg = {
                    let mut rx = p2p_in_rx.lock().await;
                    rx.recv().await
                };
                match msg {
                    Some(m) if m.topic == "qorvum-consensus" => {
                        engine_for_p2p.handle_network_msg(m.data).await;
                    }
                    Some(_) => {}
                    None => {
                        warn!("[consensus] p2p_in channel closed");
                        break;
                    }
                }
            }
        });

        // Main loop: receive TX submissions and drive consensus
        let tx_rx = self.bus.tx_receiver();
        loop {
            let sub = {
                let mut rx = tx_rx.lock().await;
                rx.recv().await
            };
            let sub = match sub {
                Some(s) => s,
                None => {
                    warn!("[consensus] tx channel closed — exiting");
                    break;
                }
            };

            let tx_id    = sub.tx_id;
            let ts       = sub.timestamp;
            let contract = sub.contract_id.clone();
            let function = sub.function_name.clone();

            let exec_result = {
                let exec = executor.read().await;
                exec.execute(
                    &contract, &function, sub.args,
                    &sub.caller_id, &sub.caller_org, sub.caller_roles,
                    tx_id, ts, sub.verified,
                )
            };

            let result = match exec_result {
                Ok(r) => r,
                Err(e) => { error!("[consensus] contract error: {}", e); continue; }
            };

            if result.ops.is_empty() { continue; }

            let block = match self.build_block(tx_id, ts, &contract, &function) {
                Ok(b)  => b,
                Err(e) => { error!("[consensus] build_block: {}", e); continue; }
            };

            let block_num = block.header.block_number;

            let block_data = match serde_json::to_vec(&(&block, &result.ops)) {
                Ok(d)  => d,
                Err(e) => { error!("[consensus] serialize: {}", e); continue; }
            };

            match self.engine.propose_block(block_data).await {
                Ok(_) => {
                    info!("[consensus] Block {} committed", block_num);
                    let _ = self.bus.block_committed_tx.send(BlockCommittedEvent {
                        block_num,
                        tx_count: 1,
                    });
                }
                Err(e) => error!("[consensus] propose_block #{}: {}", block_num, e),
            }
        }

        info!("[consensus] Role exiting");
    }

    fn build_block(
        &self,
        tx_id:    [u8; 32],
        ts:       u64,
        contract: &str,
        function: &str,
    ) -> anyhow::Result<qorvum_ledger::block::Block> {
        let latest = self.store.get_latest_block_num()
            .map_err(|e| anyhow::anyhow!(e.to_string()))?
            .unwrap_or(0);
        let prev_hash = self.store.get_block(latest)
            .map_err(|e| anyhow::anyhow!(e.to_string()))?
            .map(|b| b.compute_hash())
            .unwrap_or([0u8; 32]);

        let creator_sig = EndorsementSig { algorithm: "dilithium3".into(), bytes: vec![] };
        let tx = Transaction {
            tx_id,
            channel_id:    "main-channel".to_string(),
            contract_id:   contract.to_string(),
            function_name: function.to_string(),
            args:          serde_json::Value::Null,
            creator_pub_key: vec![],
            creator_sig:   creator_sig.clone(),
            endorsements:  vec![],
            nonce:         [0u8; 32],
            timestamp:     ts,
        };

        let mut builder = BlockBuilder::new(
            "main-channel".to_string(),
            latest + 1,
            prev_hash,
            "DefaultMSP".into(),
            vec![],
            creator_sig,
        );
        builder.add_transaction(tx);
        Ok(builder.build())
    }
}