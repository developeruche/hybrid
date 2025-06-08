#![allow(unused_mut)]
//! Hybrid payload builder, responsible for building blocks containing EVM tx and RISC-V tx
use alloy_evm::{
    block::{
        BlockExecutionError, BlockExecutor, BlockExecutorFactory, BlockExecutorFor, ExecutableTx,
        OnStateHook,
    },
    eth::{EthBlockExecutionCtx, EthBlockExecutor},
    Database, Evm, EvmEnv,
};
use hybrid_evm::eth_hybrid_evm::EthHybridEvm;
use reth::{
    api::{ConfigureEvm, FullNodeTypes, NextBlockEnvAttributes, NodeTypes, PayloadTypes},
    builder::{components::PayloadBuilderBuilder, BuilderContext},
    chainspec::ChainSpec,
    payload::{EthBuiltPayload, EthPayloadBuilderAttributes},
    primitives::{Header, SealedBlock, SealedHeader},
    providers::BlockExecutionResult,
    revm::{
        context::{result::ExecutionResult, TxEnv},
        primitives::hardfork::SpecId,
        State,
    },
    rpc::types::engine::PayloadAttributes,
    transaction_pool::{PoolTransaction, TransactionPool},
};
use reth_ethereum::{
    evm::{primitives::InspectorFor, EthBlockAssembler, EthEvmConfig, RethReceiptBuilder},
    node::node::EthereumPayloadBuilder,
    EthPrimitives, Receipt, TransactionSigned,
};
use std::sync::Arc;

use crate::factory::HybridEvmFactory;

/// Builds a regular ethereum block executor that uses the custom EVM.
#[derive(Debug, Default, Clone)]
#[non_exhaustive]
pub struct HybridPayloadBuilder {
    inner: EthereumPayloadBuilder,
}

impl<Types, Node, Pool> PayloadBuilderBuilder<Node, Pool> for HybridPayloadBuilder
where
    Types: NodeTypes<ChainSpec = ChainSpec, Primitives = EthPrimitives>,
    Node: FullNodeTypes<Types = Types>,
    Pool: TransactionPool<Transaction: PoolTransaction<Consensus = TransactionSigned>>
        + Unpin
        + 'static,
    Types::Payload: PayloadTypes<
        BuiltPayload = EthBuiltPayload,
        PayloadAttributes = PayloadAttributes,
        PayloadBuilderAttributes = EthPayloadBuilderAttributes,
    >,
{
    type PayloadBuilder = reth_ethereum_payload_builder::EthereumPayloadBuilder<
        Pool,
        Node::Provider,
        HybridEvmConfig<HybridEvmFactory>,
    >;

    async fn build_payload_builder(
        self,
        ctx: &BuilderContext<Node>,
        pool: Pool,
    ) -> eyre::Result<Self::PayloadBuilder> {
        let evm_config = HybridEvmConfig {
            inner: EthEvmConfig::<HybridEvmFactory>::new_with_evm_factory(
                ctx.chain_spec(),
                HybridEvmFactory::default(),
            ),
        };
        self.inner.build(evm_config, ctx, pool)
    }
}

#[derive(Debug, Clone)]
pub struct HybridEvmConfig<EvmFactory = HybridEvmFactory> {
    pub inner: EthEvmConfig<EvmFactory>,
}

impl BlockExecutorFactory for HybridEvmConfig {
    type EvmFactory = HybridEvmFactory;
    type ExecutionCtx<'a> = EthBlockExecutionCtx<'a>;
    type Transaction = TransactionSigned;
    type Receipt = Receipt;

    fn evm_factory(&self) -> &Self::EvmFactory {
        self.inner.evm_factory()
    }

    fn create_executor<'a, DB, I>(
        &'a self,
        evm: EthHybridEvm<&'a mut State<DB>, I>,
        ctx: EthBlockExecutionCtx<'a>,
    ) -> impl BlockExecutorFor<'a, Self, DB, I>
    where
        DB: Database + 'a,
        I: InspectorFor<Self, &'a mut State<DB>> + 'a,
    {
        HybridBlockExecutor {
            inner: EthBlockExecutor::new(
                evm,
                ctx,
                self.inner.chain_spec(),
                self.inner.executor_factory.receipt_builder(),
            ),
        }
    }
}

impl ConfigureEvm for HybridEvmConfig {
    type Primitives = <EthEvmConfig as ConfigureEvm>::Primitives;
    type Error = <EthEvmConfig as ConfigureEvm>::Error;
    type NextBlockEnvCtx = <EthEvmConfig as ConfigureEvm>::NextBlockEnvCtx;
    type BlockExecutorFactory = Self;
    type BlockAssembler = EthBlockAssembler<ChainSpec>;

    fn block_executor_factory(&self) -> &Self::BlockExecutorFactory {
        self
    }

    fn block_assembler(&self) -> &Self::BlockAssembler {
        self.inner.block_assembler()
    }

    fn evm_env(&self, header: &Header) -> EvmEnv<SpecId> {
        self.inner.evm_env(header)
    }

    fn next_evm_env(
        &self,
        parent: &Header,
        attributes: &NextBlockEnvAttributes,
    ) -> Result<EvmEnv<SpecId>, Self::Error> {
        self.inner.next_evm_env(parent, attributes)
    }

    fn context_for_block<'a>(&self, block: &'a SealedBlock) -> EthBlockExecutionCtx<'a> {
        self.inner.context_for_block(block)
    }

    fn context_for_next_block(
        &self,
        parent: &SealedHeader,
        attributes: Self::NextBlockEnvCtx,
    ) -> EthBlockExecutionCtx<'_> {
        self.inner.context_for_next_block(parent, attributes)
    }
}

pub struct HybridBlockExecutor<'a, Evm> {
    /// Inner Ethereum execution strategy.
    inner: EthBlockExecutor<'a, Evm, &'a Arc<ChainSpec>, &'a RethReceiptBuilder>,
}

impl<'db, DB, E> BlockExecutor for HybridBlockExecutor<'_, E>
where
    DB: Database + 'db,
    E: Evm<DB = &'db mut State<DB>, Tx = TxEnv>,
{
    type Transaction = TransactionSigned;
    type Receipt = Receipt;
    type Evm = E;

    fn apply_pre_execution_changes(&mut self) -> Result<(), BlockExecutionError> {
        self.inner.apply_pre_execution_changes()
    }

    fn execute_transaction_with_result_closure(
        &mut self,
        tx: impl ExecutableTx<Self>,
        f: impl FnOnce(&ExecutionResult<<Self::Evm as Evm>::HaltReason>),
    ) -> Result<u64, BlockExecutionError> {
        self.inner.execute_transaction_with_result_closure(tx, f)
    }

    fn finish(mut self) -> Result<(Self::Evm, BlockExecutionResult<Receipt>), BlockExecutionError> {
        self.inner.finish()
    }

    fn set_state_hook(&mut self, _hook: Option<Box<dyn OnStateHook>>) {
        self.inner.set_state_hook(_hook)
    }

    fn evm_mut(&mut self) -> &mut Self::Evm {
        self.inner.evm_mut()
    }

    fn evm(&self) -> &Self::Evm {
        self.inner.evm()
    }
}
