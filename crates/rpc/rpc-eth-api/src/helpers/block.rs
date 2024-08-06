//! Database access for `eth_` block RPC methods. Loads block and receipt data w.r.t. network.

use std::sync::Arc;

use futures::Future;
use reth_primitives::{BlockId, Receipt, SealedBlock, SealedBlockWithSenders, TransactionMeta};
use reth_provider::{BlockIdReader, BlockReader, BlockReaderIdExt, HeaderProvider};
<<<<<<< HEAD
use reth_rpc_eth_types::{EthApiError, EthResult, EthStateCache, ReceiptBuilder};
use reth_rpc_types::{AnyTransactionReceipt, Header, Index, RichBlock};
use reth_rpc_types_compat::block::{from_block, uncle_block_from_header};

=======
use reth_rpc_eth_types::{EthApiError, EthStateCache, ReceiptBuilder};
use reth_rpc_types::{AnyTransactionReceipt, Header, Index, RichBlock};
use reth_rpc_types_compat::block::{from_block, uncle_block_from_header};

use crate::FromEthApiError;

>>>>>>> c4b5f5e9c9a88783b2def3ab1cc880b8d41867e1
use super::{LoadPendingBlock, LoadReceipt, SpawnBlocking};

/// Block related functions for the [`EthApiServer`](crate::EthApiServer) trait in the
/// `eth_` namespace.
pub trait EthBlocks: LoadBlock {
    /// Returns a handle for reading data from disk.
    ///
    /// Data access in default (L1) trait method implementations.
    fn provider(&self) -> impl HeaderProvider;

    /// Returns the block header for the given block id.
    fn rpc_block_header(
        &self,
        block_id: BlockId,
<<<<<<< HEAD
    ) -> impl Future<Output = EthResult<Option<Header>>> + Send
=======
    ) -> impl Future<Output = Result<Option<Header>, Self::Error>> + Send
>>>>>>> c4b5f5e9c9a88783b2def3ab1cc880b8d41867e1
    where
        Self: LoadPendingBlock + SpawnBlocking,
    {
        async move { Ok(self.rpc_block(block_id, false).await?.map(|block| block.inner.header)) }
    }

    /// Returns the populated rpc block object for the given block id.
    ///
    /// If `full` is true, the block object will contain all transaction objects, otherwise it will
    /// only contain the transaction hashes.
    fn rpc_block(
        &self,
        block_id: BlockId,
        full: bool,
<<<<<<< HEAD
    ) -> impl Future<Output = EthResult<Option<RichBlock>>> + Send
=======
    ) -> impl Future<Output = Result<Option<RichBlock>, Self::Error>> + Send
>>>>>>> c4b5f5e9c9a88783b2def3ab1cc880b8d41867e1
    where
        Self: LoadPendingBlock + SpawnBlocking,
    {
        async move {
            let block = match self.block_with_senders(block_id).await? {
                Some(block) => block,
                None => return Ok(None),
            };
            let block_hash = block.hash();
            let total_difficulty = EthBlocks::provider(self)
<<<<<<< HEAD
                .header_td_by_number(block.number)?
                .ok_or(EthApiError::UnknownBlockNumber)?;
            let block =
                from_block(block.unseal(), total_difficulty, full.into(), Some(block_hash))?;
=======
                .header_td_by_number(block.number)
                .map_err(Self::Error::from_eth_err)?
                .ok_or(EthApiError::UnknownBlockNumber)?;
            let block = from_block(block.unseal(), total_difficulty, full.into(), Some(block_hash))
                .map_err(Self::Error::from_eth_err)?;
>>>>>>> c4b5f5e9c9a88783b2def3ab1cc880b8d41867e1
            Ok(Some(block.into()))
        }
    }

    /// Returns the number transactions in the given block.
    ///
    /// Returns `None` if the block does not exist
    fn block_transaction_count(
        &self,
        block_id: BlockId,
<<<<<<< HEAD
    ) -> impl Future<Output = EthResult<Option<usize>>> + Send {
        async move {
            if block_id.is_pending() {
                // Pending block can be fetched directly without need for caching
                return Ok(LoadBlock::provider(self).pending_block()?.map(|block| block.body.len()));
            }

            let block_hash = match LoadBlock::provider(self).block_hash_for_id(block_id)? {
=======
    ) -> impl Future<Output = Result<Option<usize>, Self::Error>> + Send {
        async move {
            if block_id.is_pending() {
                // Pending block can be fetched directly without need for caching
                return Ok(LoadBlock::provider(self)
                    .pending_block()
                    .map_err(Self::Error::from_eth_err)?
                    .map(|block| block.body.len()))
            }

            let block_hash = match LoadBlock::provider(self)
                .block_hash_for_id(block_id)
                .map_err(Self::Error::from_eth_err)?
            {
>>>>>>> c4b5f5e9c9a88783b2def3ab1cc880b8d41867e1
                Some(block_hash) => block_hash,
                None => return Ok(None),
            };

<<<<<<< HEAD
            Ok(self.cache().get_block_transactions(block_hash).await?.map(|txs| txs.len()))
=======
            Ok(self
                .cache()
                .get_block_transactions(block_hash)
                .await
                .map_err(Self::Error::from_eth_err)?
                .map(|txs| txs.len()))
>>>>>>> c4b5f5e9c9a88783b2def3ab1cc880b8d41867e1
        }
    }

    /// Helper function for `eth_getBlockReceipts`.
    ///
    /// Returns all transaction receipts in block, or `None` if block wasn't found.
    fn block_receipts(
        &self,
        block_id: BlockId,
<<<<<<< HEAD
    ) -> impl Future<Output = EthResult<Option<Vec<AnyTransactionReceipt>>>> + Send
=======
    ) -> impl Future<Output = Result<Option<Vec<AnyTransactionReceipt>>, Self::Error>> + Send
>>>>>>> c4b5f5e9c9a88783b2def3ab1cc880b8d41867e1
    where
        Self: LoadReceipt,
    {
        async move {
            if let Some((block, receipts)) = self.load_block_and_receipts(block_id).await? {
                let block_number = block.number;
                let base_fee = block.base_fee_per_gas;
                let block_hash = block.hash();
                let excess_blob_gas = block.excess_blob_gas;
                let timestamp = block.timestamp;
                let block = block.unseal();

                let receipts = block
                    .body
                    .into_iter()
                    .zip(receipts.iter())
                    .enumerate()
                    .map(|(idx, (tx, receipt))| {
                        let meta = TransactionMeta {
                            tx_hash: tx.hash,
                            index: idx as u64,
                            block_hash,
                            block_number,
                            base_fee,
                            excess_blob_gas,
                            timestamp,
                        };

                        ReceiptBuilder::new(&tx, meta, receipt, &receipts)
                            .map(|builder| builder.build())
<<<<<<< HEAD
                    })
                    .collect::<EthResult<Vec<_>>>();
                return receipts.map(Some);
=======
                            .map_err(Self::Error::from_eth_err)
                    })
                    .collect::<Result<Vec<_>, Self::Error>>();
                return receipts.map(Some)
>>>>>>> c4b5f5e9c9a88783b2def3ab1cc880b8d41867e1
            }

            Ok(None)
        }
    }

    /// Helper method that loads a bock and all its receipts.
    fn load_block_and_receipts(
        &self,
        block_id: BlockId,
<<<<<<< HEAD
    ) -> impl Future<Output = EthResult<Option<(SealedBlock, Arc<Vec<Receipt>>)>>> + Send
=======
    ) -> impl Future<Output = Result<Option<(SealedBlock, Arc<Vec<Receipt>>)>, Self::Error>> + Send
>>>>>>> c4b5f5e9c9a88783b2def3ab1cc880b8d41867e1
    where
        Self: LoadReceipt,
    {
        async move {
            if block_id.is_pending() {
                return Ok(LoadBlock::provider(self)
<<<<<<< HEAD
                    .pending_block_and_receipts()?
                    .map(|(sb, receipts)| (sb, Arc::new(receipts))));
            }

            if let Some(block_hash) = LoadBlock::provider(self).block_hash_for_id(block_id)? {
                return Ok(LoadReceipt::cache(self).get_block_and_receipts(block_hash).await?);
=======
                    .pending_block_and_receipts()
                    .map_err(Self::Error::from_eth_err)?
                    .map(|(sb, receipts)| (sb, Arc::new(receipts))))
            }

            if let Some(block_hash) = LoadBlock::provider(self)
                .block_hash_for_id(block_id)
                .map_err(Self::Error::from_eth_err)?
            {
                return LoadReceipt::cache(self)
                    .get_block_and_receipts(block_hash)
                    .await
                    .map_err(Self::Error::from_eth_err)
>>>>>>> c4b5f5e9c9a88783b2def3ab1cc880b8d41867e1
            }

            Ok(None)
        }
    }

    /// Returns uncle headers of given block.
    ///
    /// Returns an empty vec if there are none.
<<<<<<< HEAD
    fn ommers(&self, block_id: BlockId) -> EthResult<Option<Vec<reth_primitives::Header>>> {
        Ok(LoadBlock::provider(self).ommers_by_id(block_id)?)
=======
    fn ommers(
        &self,
        block_id: BlockId,
    ) -> Result<Option<Vec<reth_primitives::Header>>, Self::Error> {
        LoadBlock::provider(self).ommers_by_id(block_id).map_err(Self::Error::from_eth_err)
>>>>>>> c4b5f5e9c9a88783b2def3ab1cc880b8d41867e1
    }

    /// Returns uncle block at given index in given block.
    ///
    /// Returns `None` if index out of range.
    fn ommer_by_block_and_index(
        &self,
        block_id: BlockId,
        index: Index,
<<<<<<< HEAD
    ) -> impl Future<Output = EthResult<Option<RichBlock>>> + Send {
        async move {
            let uncles = if block_id.is_pending() {
                // Pending block can be fetched directly without need for caching
                LoadBlock::provider(self).pending_block()?.map(|block| block.ommers)
            } else {
                LoadBlock::provider(self).ommers_by_id(block_id)?
=======
    ) -> impl Future<Output = Result<Option<RichBlock>, Self::Error>> + Send {
        async move {
            let uncles = if block_id.is_pending() {
                // Pending block can be fetched directly without need for caching
                LoadBlock::provider(self)
                    .pending_block()
                    .map_err(Self::Error::from_eth_err)?
                    .map(|block| block.ommers)
            } else {
                LoadBlock::provider(self)
                    .ommers_by_id(block_id)
                    .map_err(Self::Error::from_eth_err)?
>>>>>>> c4b5f5e9c9a88783b2def3ab1cc880b8d41867e1
            }
            .unwrap_or_default();

            let index = usize::from(index);
            let uncle =
                uncles.into_iter().nth(index).map(|header| uncle_block_from_header(header).into());
            Ok(uncle)
        }
    }
}

/// Loads a block from database.
///
/// Behaviour shared by several `eth_` RPC methods, not exclusive to `eth_` blocks RPC methods.
pub trait LoadBlock: LoadPendingBlock + SpawnBlocking {
    // Returns a handle for reading data from disk.
    ///
    /// Data access in default (L1) trait method implementations.
    fn provider(&self) -> impl BlockReaderIdExt;

    /// Returns a handle for reading data from memory.
    ///
    /// Data access in default (L1) trait method implementations.
    fn cache(&self) -> &EthStateCache;

    /// Returns the block object for the given block id.
    fn block(
        &self,
        block_id: BlockId,
<<<<<<< HEAD
    ) -> impl Future<Output = EthResult<Option<SealedBlock>>> + Send {
=======
    ) -> impl Future<Output = Result<Option<SealedBlock>, Self::Error>> + Send {
>>>>>>> c4b5f5e9c9a88783b2def3ab1cc880b8d41867e1
        async move {
            self.block_with_senders(block_id)
                .await
                .map(|maybe_block| maybe_block.map(|block| block.block))
        }
    }

    /// Returns the block object for the given block id.
    fn block_with_senders(
        &self,
        block_id: BlockId,
<<<<<<< HEAD
    ) -> impl Future<Output = EthResult<Option<SealedBlockWithSenders>>> + Send {
        async move {
            if block_id.is_pending() {
                // Pending block can be fetched directly without need for caching
                let maybe_pending =
                    LoadPendingBlock::provider(self).pending_block_with_senders()?;
=======
    ) -> impl Future<Output = Result<Option<SealedBlockWithSenders>, Self::Error>> + Send {
        async move {
            if block_id.is_pending() {
                // Pending block can be fetched directly without need for caching
                let maybe_pending = LoadPendingBlock::provider(self)
                    .pending_block_with_senders()
                    .map_err(Self::Error::from_eth_err)?;
>>>>>>> c4b5f5e9c9a88783b2def3ab1cc880b8d41867e1
                return if maybe_pending.is_some() {
                    Ok(maybe_pending)
                } else {
                    self.local_pending_block().await
<<<<<<< HEAD
                };
            }

            let block_hash = match LoadPendingBlock::provider(self).block_hash_for_id(block_id)? {
=======
                }
            }

            let block_hash = match LoadPendingBlock::provider(self)
                .block_hash_for_id(block_id)
                .map_err(Self::Error::from_eth_err)?
            {
>>>>>>> c4b5f5e9c9a88783b2def3ab1cc880b8d41867e1
                Some(block_hash) => block_hash,
                None => return Ok(None),
            };

<<<<<<< HEAD
            Ok(self.cache().get_sealed_block_with_senders(block_hash).await?)
=======
            self.cache()
                .get_sealed_block_with_senders(block_hash)
                .await
                .map_err(Self::Error::from_eth_err)
>>>>>>> c4b5f5e9c9a88783b2def3ab1cc880b8d41867e1
        }
    }
}
