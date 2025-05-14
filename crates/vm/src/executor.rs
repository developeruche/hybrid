//! Executor enbeding the custom VM

use reth::{
    api::{FullNodeTypes, NodeTypes},
    builder::{components::ExecutorBuilder, BuilderContext},
    chainspec::ChainSpec,
};
use reth_ethereum::{evm::EthEvmConfig, EthPrimitives};

use crate::factory::VmFactory;

/// Builds a regular ethereum block executor that uses the custom EVM.
#[derive(Debug, Default, Clone, Copy)]
#[non_exhaustive]
pub struct VmExecutorBuilder;

impl<Node> ExecutorBuilder<Node> for VmExecutorBuilder
where
    Node: FullNodeTypes<Types: NodeTypes<ChainSpec = ChainSpec, Primitives = EthPrimitives>>,
{
    type EVM = EthEvmConfig<VmFactory>;

    async fn build_evm(self, ctx: &BuilderContext<Node>) -> eyre::Result<Self::EVM> {
        let evm_config = EthEvmConfig::new_with_evm_factory(ctx.chain_spec(), VmFactory::default());
        Ok(evm_config)
    }
}
