//! Module houses the VM sandbox for this node (EVM and r55-(RISC-V VM))
use constants::obtain_specs;
use executor::VmExecutorBuilder;
use reth::{
    args::RpcServerArgs,
    builder::{NodeBuilder, NodeConfig},
    tasks::TaskManager,
};
use reth_ethereum::node::{node::EthereumAddOns, EthereumNode};
use reth_tracing::{RethTracer, Tracer};

pub mod constants;
pub mod executor;
pub mod factory;
pub mod hybrid_evm;

pub async fn run_node(is_dev: bool) -> Result<(), eyre::Error> {
    let _guard = RethTracer::new().init().map_err(|e| anyhow::anyhow!(e));

    let tasks = TaskManager::current();
    let spec = obtain_specs();

    let node_config = if is_dev {
        NodeConfig::test()
            .dev()
            .with_rpc(RpcServerArgs::default().with_http())
            .with_chain(spec)
    } else {
        NodeConfig::test()
            .with_rpc(RpcServerArgs::default().with_http())
            .with_chain(spec)
    };

    let handle = NodeBuilder::new(node_config)
        .testing_node(tasks.executor())
        // configure the node with regular ethereum types
        .with_types::<EthereumNode>()
        // use default ethereum components but with our executor
        .with_components(EthereumNode::components().executor(VmExecutorBuilder::default()))
        .with_add_ons(EthereumAddOns::default())
        .launch()
        .await
        .unwrap();

    println!("Node started");

    handle.node_exit_future.await
}
