//! Tambahan untuk handlers.rs yang sudah ada.
//! Paste fungsi commit_block ini menggantikan yang lama — bedanya
//! sekarang setelah commit, dia broadcast SSE event ke semua frontend.

use std::sync::Arc;
use qorvum_ledger::{
    block::{Block, BlockBuilder, EndorsementSig, Transaction},
    store::RecordOp,
};
use crate::error::ApiError;
use crate::state::AppState;

/// Build block dan commit — sama seperti sebelumnya tapi sekarang
/// broadcast BlockEvent dan TxEvent ke SSE broadcaster setelah commit.
pub async fn commit_block_with_events(
    state:    &AppState,
    tx_id:    [u8; 32],
    ts:       u64,
    contract: &str,
    function: &str,
    args:     serde_json::Value,
    ops:      Vec<RecordOp>,
    caller_id: &str,
) -> Result<u64, ApiError> {
    let (block, block_num) = build_block(state, tx_id, ts, contract, function, args, &ops).await?;

    if let Some(ref engine) = state.consensus {
        let block_data = serde_json::to_vec(&(&block, &ops))
            .map_err(|e| ApiError::Internal(format!("serialize block_data: {e}")))?;

        engine.propose_block(block_data).await
            .map_err(|e| ApiError::Internal(format!("consensus: {e}")))?;
    } else {
        state.store.commit_block(&block, ops)
            .map_err(|e| ApiError::Internal(e.to_string()))?;
    }

    // Broadcast SSE events setelah commit berhasil
    state.broadcaster.block_committed(block_num, 1);
    state.broadcaster.tx_committed(
        hex::encode(tx_id),
        block_num,
        contract.to_string(),
        function.to_string(),
        caller_id.to_string(),
        true,
    );

    Ok(block_num)
}

async fn build_block(
    state:    &AppState,
    tx_id:    [u8; 32],
    ts:       u64,
    contract: &str,
    function: &str,
    args:     serde_json::Value,
    _ops:     &[RecordOp],
) -> Result<(Block, u64), ApiError> {
    let latest = state.store.get_latest_block_num()
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .unwrap_or(0);
    let prev_hash = state.store.get_block(latest)
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .map(|b| b.compute_hash())
        .unwrap_or([0u8; 32]);

    let creator_sig = EndorsementSig { algorithm: "dev-mode".into(), bytes: vec![] };
    let tx = Transaction {
        tx_id,
        channel_id:    state.channel_id.clone(),
        contract_id:   contract.to_string(),
        function_name: function.to_string(),
        args,
        creator_pub_key: vec![],
        creator_sig:    creator_sig.clone(),
        endorsements:   vec![],
        nonce:          [0u8; 32],
        timestamp:      ts,
    };

    let mut builder = BlockBuilder::new(
        state.channel_id.clone(),
        latest + 1,
        prev_hash,
        "DefaultMSP".into(),
        vec![],
        creator_sig,
    );
    builder.add_transaction(tx);
    let block     = builder.build();
    let block_num = block.header.block_number;

    Ok((block, block_num))
}