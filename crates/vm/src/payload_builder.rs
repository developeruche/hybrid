//! Hybrid payload builder, responsible for building blocks containing EVM tx and RISC-V tx
use reth::{
    api::{FullNodeTypes, NodeTypes, PayloadTypes},
    builder::{components::PayloadBuilderBuilder, BuilderContext},
    chainspec::ChainSpec,
    payload::{EthBuiltPayload, EthPayloadBuilderAttributes},
    rpc::types::engine::PayloadAttributes,
    transaction_pool::{PoolTransaction, TransactionPool},
};
use reth_ethereum::{
    evm::EthEvmConfig, node::node::EthereumPayloadBuilder, EthPrimitives, TransactionSigned,
};

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
        EthEvmConfig<HybridEvmFactory>,
    >;

    async fn build_payload_builder(
        self,
        ctx: &BuilderContext<Node>,
        pool: Pool,
    ) -> eyre::Result<Self::PayloadBuilder> {
        let evm_config =
            EthEvmConfig::new_with_evm_factory(ctx.chain_spec(), HybridEvmFactory::default());
        self.inner.build(evm_config, ctx, pool)
    }
}
