//! VM factory related ops
use alloy_evm::{eth::EthEvmContext, precompiles::PrecompilesMap, EvmFactory};
use reth_ethereum::evm::{
    primitives::{Database, EvmEnv},
    revm::{
        context::{Context, TxEnv},
        context_interface::result::{EVMError, HaltReason},
        handler::EthPrecompiles,
        inspector::{Inspector, NoOpInspector},
        interpreter::interpreter::EthInterpreter,
        primitives::hardfork::SpecId,
        MainBuilder, MainContext,
    },
    EthEvm,
};
use reth_tracing::tracing::instrument::WithSubscriber;

#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct VmFactory;

impl EvmFactory for VmFactory {
    type Evm<DB: Database, I: Inspector<EthEvmContext<DB>, EthInterpreter>> =
        EthEvm<DB, I, Self::Precompiles>;
    type Tx = TxEnv;
    type Error<DBError: core::error::Error + Send + Sync + 'static> = EVMError<DBError>;
    type HaltReason = HaltReason;
    type Context<DB: Database> = EthEvmContext<DB>;
    type Spec = SpecId;
    type Precompiles = PrecompilesMap;

    fn create_evm<DB: Database>(&self, db: DB, input: EvmEnv) -> Self::Evm<DB, NoOpInspector> {
        let evm = Context::mainnet()
            .with_db(db)
            .with_cfg(input.cfg_env)
            .with_block(input.block_env)
            .build_mainnet_with_inspector(NoOpInspector {})
            .with_precompiles(PrecompilesMap::from_static(
                EthPrecompiles::default().precompiles,
            ));

        // let evm = HybridEvm::new(Context::mainnet()
        //     .with_db(db)
        //     .with_cfg(input.cfg_env)
        //     .with_block(input.block_env), NoOpInspector {});

        EthEvm::new(evm, false)
    }

    fn create_evm_with_inspector<DB: Database, I: Inspector<Self::Context<DB>, EthInterpreter>>(
        &self,
        db: DB,
        input: EvmEnv,
        inspector: I,
    ) -> Self::Evm<DB, I> {
        EthEvm::new(
            self.create_evm(db, input)
                .into_inner()
                .with_inspector(inspector),
            true,
        )
    }
}
