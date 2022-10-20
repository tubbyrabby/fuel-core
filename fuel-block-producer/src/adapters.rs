use crate::{
    adapters::transaction_selector::select_transactions,
    ports::TxPool,
};
use fuel_core_interfaces::{
    common::fuel_tx::ConsensusParameters,
    model::{
        ArcTx,
        BlockHeight,
    },
    txpool::Sender,
};

pub mod transaction_selector;

pub struct TxPoolAdapter {
    pub sender: Sender,
    pub consensus_params: ConsensusParameters,
}

#[async_trait::async_trait]
impl TxPool for TxPoolAdapter {
    async fn get_includable_txs(
        &self,
        _block_height: BlockHeight,
        max_gas: u64,
    ) -> anyhow::Result<Vec<ArcTx>> {
        let includable_txs =
            select_transactions(self.sender.includable().await?, max_gas);

        Ok(includable_txs)
    }
}
