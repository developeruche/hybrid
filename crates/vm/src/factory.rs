//! VM factory related ops
use reth::revm::{
    context::{
        result::{EVMError, HaltReason},
        TxEnv,
    },
    handler::EthPrecompiles,
    inspector::NoOpInspector,
    interpreter::interpreter::EthInterpreter,
    primitives::hardfork::SpecId,
    Context, Database, Inspector, MainBuilder, MainContext,
};
use reth_ethereum::evm::{
    primitives::{eth::EthEvmContext, EvmEnv, EvmFactory},
    EthEvm,
};

/// Hybrid EVM configuration.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct HybridEvmFactory;

impl EvmFactory for HybridEvmFactory {
    type Evm<DB: Database, I: Inspector<EthEvmContext<DB>, EthInterpreter>>
        = EthEvm<DB, I, EthPrecompiles>
    where
        <DB as Database>::Error: Send + Sync + 'static;
    type Tx = TxEnv;
    type Error<DBError: core::error::Error + Send + Sync + 'static> = EVMError<DBError>;
    type HaltReason = HaltReason;
    type Context<DB: Database>
        = EthEvmContext<DB>
    where
        <DB as Database>::Error: Send + Sync + 'static;
    type Spec = SpecId;

    fn create_evm<DB: Database>(&self, db: DB, input: EvmEnv) -> Self::Evm<DB, NoOpInspector>
    where
        <DB as Database>::Error: Send + Sync + 'static,
    {
        let evm = Context::mainnet()
            .with_db(db)
            .with_cfg(input.cfg_env)
            .with_block(input.block_env)
            .build_mainnet_with_inspector(NoOpInspector {})
            .with_precompiles(EthPrecompiles::default());

        EthEvm::new(evm, false)
    }

    fn create_evm_with_inspector<DB: Database, I: Inspector<Self::Context<DB>, EthInterpreter>>(
        &self,
        db: DB,
        input: EvmEnv,
        inspector: I,
    ) -> Self::Evm<DB, I>
    where
        <DB as Database>::Error: Send + Sync + 'static,
    {
        EthEvm::new(
            self.create_evm(db, input)
                .into_inner()
                .with_inspector(inspector),
            true,
        )
    }
}
