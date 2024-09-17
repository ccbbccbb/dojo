// TODO: remove the messaging feature flag
// TODO: move the tasks to a separate module

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use futures::channel::mpsc::Receiver;
use futures::stream::{Fuse, Stream, StreamExt};
use katana_executor::ExecutorFactory;
use katana_pool::{TransactionPool, TxPool};
use katana_primitives::transaction::ExecutableTxWithHash;
use katana_primitives::Felt;
use tracing::{error, info};

use self::block_producer::BlockProducer;
use self::metrics::BlockProducerMetrics;

pub mod block_producer;
#[cfg(feature = "messaging")]
pub mod messaging;
mod metrics;

pub(crate) const LOG_TARGET: &str = "node";

/// The type that drives the blockchain's state
///
/// This task is basically an endless future that continuously polls the miner which returns
/// transactions for the next block, then those transactions are handed off to the [BlockProducer]
/// to construct a new block.
#[must_use = "BlockProductionTask does nothing unless polled"]
#[allow(missing_debug_implementations)]
pub struct BlockProductionTask<EF, P = TxPool>
where
    EF: ExecutorFactory,
    P: TransactionPool,
    P::Transaction: Into<ExecutableTxWithHash>,
{
    /// creates new blocks
    pub(crate) block_producer: Arc<BlockProducer<EF>>,
    /// the miner responsible to select transactions from the `pool´
    pub(crate) miner: TransactionMiner<P>,
    /// the pool that holds all transactions
    pub(crate) pool: P,
    /// Metrics for recording the service operations
    metrics: BlockProducerMetrics,
}

impl<EF, P> BlockProductionTask<EF, P>
where
    EF: ExecutorFactory,
    P: TransactionPool,
    P::Transaction: Into<ExecutableTxWithHash>,
{
    pub fn new(
        pool: P,
        miner: TransactionMiner<P>,
        block_producer: Arc<BlockProducer<EF>>,
    ) -> Self {
        Self { block_producer, miner, pool, metrics: BlockProducerMetrics::default() }
    }
}

impl<EF, P> Future for BlockProductionTask<EF, P>
where
    EF: ExecutorFactory,
    P: TransactionPool + Unpin,
    P::Transaction: Into<ExecutableTxWithHash>,
{
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();

        // this drives block production and feeds new sets of ready transactions to the block
        // producer
        loop {
            while let Poll::Ready(Some(res)) = this.block_producer.poll_next(cx) {
                match res {
                    Ok(outcome) => {
                        info!(target: LOG_TARGET, block_number = %outcome.block_number, "Mined block.");

                        let gas_used = outcome.stats.l1_gas_used;
                        let steps_used = outcome.stats.cairo_steps_used;
                        this.metrics.l1_gas_processed_total.increment(gas_used as u64);
                        this.metrics.cairo_steps_processed_total.increment(steps_used as u64);
                    }

                    Err(error) => {
                        error!(target: LOG_TARGET, %error, "Mining block.");
                    }
                }
            }

            if let Poll::Ready(pool_txs) = this.miner.poll(&this.pool, cx) {
                // miner returned a set of transaction that we then feed to the producer
                let executables = pool_txs.into_iter().map(Into::into).collect::<Vec<_>>();
                this.block_producer.queue(executables);
            } else {
                // no progress made
                break;
            }
        }

        Poll::Pending
    }
}

/// The type which takes the transaction from the pool and feeds them to the block producer.
#[derive(Debug)]
pub struct TransactionMiner<P> {
    /// stores whether there are pending transacions (if known)
    has_pending_txs: Option<bool>,
    /// Receives hashes of transactions that are ready from the pool
    rx: Fuse<Receiver<Felt>>,

    _pool: std::marker::PhantomData<P>,
}

impl<P: TransactionPool> TransactionMiner<P> {
    pub fn new(rx: Receiver<Felt>) -> Self {
        Self { rx: rx.fuse(), has_pending_txs: None, _pool: std::marker::PhantomData }
    }

    fn poll(&mut self, pool: &P, cx: &mut Context<'_>) -> Poll<Vec<P::Transaction>> {
        // drain the notification stream
        while let Poll::Ready(Some(_)) = Pin::new(&mut self.rx).poll_next(cx) {
            self.has_pending_txs = Some(true);
        }

        if self.has_pending_txs == Some(false) {
            return Poll::Pending;
        }

        // take all the transactions from the pool
        let transactions =
            pool.take_transactions().map(|tx| tx.tx.as_ref().clone()).collect::<Vec<_>>();

        if transactions.is_empty() {
            return Poll::Pending;
        }

        self.has_pending_txs = Some(false);
        Poll::Ready(transactions)
    }
}
