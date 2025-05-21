// //! VM factory related ops
// use alloy_evm::{eth::{EthEvmContext, EthHybridEvm}, hybrid_evm::evm::HybridEvm, precompiles::PrecompilesMap, EvmEnv, EvmFactory};
// use reth_ethereum::evm::{
//     primitives::{Database},
//     revm::{
//         context::{Context, TxEnv},
//         context_interface::result::{EVMError, HaltReason},
//         handler::EthPrecompiles,
//         inspector::{Inspector, NoOpInspector},
//         interpreter::interpreter::EthInterpreter,
//         primitives::hardfork::SpecId,
//         MainBuilder, MainContext,
//     },
// };
// use reth_tracing::tracing::instrument::WithSubscriber;

// #[derive(Debug, Clone, Default)]
// #[non_exhaustive]
// pub struct VmFactory;

// impl EvmFactory for VmFactory {
//     type Evm<DB: Database, I: Inspector<EthEvmContext<DB>, EthInterpreter>> =
//         EthHybridEvm<DB, I>;
//     type Tx = TxEnv;
//     type Error<DBError: core::error::Error + Send + Sync + 'static> = EVMError<DBError>;
//     type HaltReason = HaltReason;
//     type Context<DB: Database> = EthEvmContext<DB>;
//     type Spec = SpecId;
//     type Precompiles = EthPrecompiles;

//     fn create_evm<DB: Database>(&self, db: DB, input: EvmEnv) -> Self::Evm<DB, NoOpInspector> {
//         let evm = HybridEvm::new(Context::mainnet()
//             .with_db(db)
//             .with_cfg(input.cfg_env)
//             .with_block(input.block_env), NoOpInspector {});

//         EthHybridEvm::new(evm, false)
//     }

//     fn create_evm_with_inspector<DB: Database, I: Inspector<Self::Context<DB>, EthInterpreter>>(
//         &self,
//         db: DB,
//         input: EvmEnv,
//         inspector: I,
//     ) -> Self::Evm<DB, I> {
//         EthHybridEvm::new(
//             self.create_evm(db, input)
//                 .into_inner()
//                 .0.with_inspector(inspector),
//             true,
//         )
//     }
// }
