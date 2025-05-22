//! Executor enbeding the custom VM
use crate::factory::HybridEvmFactory;
use reth::{
    api::{FullNodeTypes, NodeTypes},
    builder::{components::ExecutorBuilder, BuilderContext},
    chainspec::ChainSpec,
};
use reth_ethereum::{evm::EthEvmConfig, node::BasicBlockExecutorProvider, EthPrimitives};

/// Builds a regular ethereum block executor that uses the custom EVM.
#[derive(Debug, Default, Clone, Copy)]
#[non_exhaustive]
pub struct HybridExecutorBuilder;

impl<Node> ExecutorBuilder<Node> for HybridExecutorBuilder
where
    Node: FullNodeTypes<Types: NodeTypes<ChainSpec = ChainSpec, Primitives = EthPrimitives>>,
{
    type EVM = EthEvmConfig<HybridEvmFactory>;
    type Executor = BasicBlockExecutorProvider<Self::EVM>;

    async fn build_evm(
        self,
        ctx: &BuilderContext<Node>,
    ) -> eyre::Result<(Self::EVM, Self::Executor)> {
        let evm_config =
            EthEvmConfig::new_with_evm_factory(ctx.chain_spec(), HybridEvmFactory::default());
        Ok((
            evm_config.clone(),
            BasicBlockExecutorProvider::new(evm_config),
        ))
    }
}
