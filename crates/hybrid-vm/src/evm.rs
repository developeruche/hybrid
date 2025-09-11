use reth::revm::{
    context::{
        Block, BlockEnv, Cfg, CfgEnv, ContextSetters, ContextTr, Evm, EvmData, JournalTr,
        Transaction, TxEnv,
    },
    db::EmptyDB,
    handler::{
        instructions::{EthInstructions, InstructionProvider},
        EthPrecompiles, EvmTr,
    },
    inspector::{inspect_instructions, InspectorEvmTr, JournalExt},
    interpreter::{interpreter::EthInterpreter, Interpreter, InterpreterTypes},
    Context, Inspector, Journal,
};
use serde::Serialize;

use crate::{
    execution::helper::dram_slice,
    mini_evm_coding::{deserialize_output, serialize_input, Input},
    setup::setup_from_mini_elf,
};

/// HybridEvm variant of the EVM.
pub struct HybridEvm<CTX, INSP>(
    pub Evm<CTX, INSP, EthInstructions<EthInterpreter, CTX>, EthPrecompiles>,
);

impl<CTX: ContextTr, INSP> HybridEvm<CTX, INSP> {
    pub fn new(ctx: CTX, inspector: INSP) -> Self {
        Self(Evm {
            data: EvmData {
                ctx: ctx,
                inspector,
            },
            instruction: EthInstructions::new_mainnet(),
            precompiles: EthPrecompiles::default(),
        })
    }
}

impl<CTX: ContextTr, INSP> EvmTr for HybridEvm<CTX, INSP>
where
    CTX: ContextTr,
{
    type Context = CTX;
    type Instructions = EthInstructions<EthInterpreter, CTX>;
    type Precompiles = EthPrecompiles;

    fn ctx(&mut self) -> &mut Self::Context {
        &mut self.0.data.ctx
    }

    fn ctx_ref(&self) -> &Self::Context {
        self.0.ctx_ref()
    }

    fn ctx_instructions(&mut self) -> (&mut Self::Context, &mut Self::Instructions) {
        self.0.ctx_instructions()
    }

    fn run_interpreter(
        &mut self,
        interpreter: &mut Interpreter<
            <Self::Instructions as InstructionProvider>::InterpreterTypes,
        >,
    ) -> <<Self::Instructions as InstructionProvider>::InterpreterTypes as InterpreterTypes>::Output
    {
        let context = &mut self.0.data.ctx;
        // let instructions = &self.0.instruction;

        // interpreter.run_plain(instructions.instruction_table(), context)

        let serialized_interpreter = serde_json::to_vec(interpreter).unwrap();
        let sss: Interpreter = serde_json::from_slice(&serialized_interpreter).unwrap();

        println!(
            "This is the serial interpreter: {:?}",
            serialized_interpreter
        );

        let block = BlockEnv {
            basefee: context.block().basefee(),
            beneficiary: context.block().beneficiary(),
            blob_excess_gas_and_price: context.block().blob_excess_gas_and_price(),
            difficulty: context.block().difficulty(),
            gas_limit: context.block().gas_limit(),
            number: context.block().number(),
            prevrandao: context.block().prevrandao(),
            timestamp: context.block().timestamp(),
        };

        let mut cfg = CfgEnv::new();
        cfg.chain_id = context.cfg().chain_id();

        let mut tx = TxEnv::default();
        tx.access_list = Default::default();
        tx.authorization_list = Default::default();
        tx.blob_hashes = context.tx().blob_versioned_hashes().to_vec();
        tx.caller = context.tx().caller();
        tx.chain_id = context.tx().chain_id();
        tx.data = context.tx().input().clone();
        tx.gas_limit = context.tx().gas_limit();
        tx.gas_price = context.tx().gas_price();
        tx.gas_priority_fee = context.tx().max_priority_fee_per_gas();
        tx.kind = context.tx().kind();
        tx.max_fee_per_blob_gas = context.tx().max_fee_per_blob_gas();
        tx.nonce = context.tx().nonce();
        tx.tx_type = context.tx().tx_type();
        tx.value = context.tx().value();

        let db = EmptyDB::new();
        let c: Context = Context {
            block: block,
            cfg: cfg,
            chain: (),
            error: Ok(()),
            journaled_state: Journal::new(db),
            tx: tx,
        };

        let emu_input = serialize_input(&serialized_interpreter, &c).unwrap();

        println!("Emu Input: {:?}", emu_input);

        let mini_evm_bin: &[u8] = include_bytes!("../mini-interpreter");

        let mut emulator = match setup_from_mini_elf(mini_evm_bin, &emu_input) {
            Ok(emulator) => emulator,
            Err(err) => {
                // TODO:: handle this gracefully
                panic!("Error occurred setting up emulator")
            }
        };

        let return_res = emulator.estart();

        match return_res {
            Ok(_) => (),
            Err(err) => {
                println!("Emulator Error Occured: {:?}", err)
            }
        }

        let interpreter_output_size: u64 = emulator.cpu.xregs.read(10);
        println!("Interpreter output size: {}", interpreter_output_size);

        let raw_output = dram_slice(&mut emulator, 0x8000_0000, interpreter_output_size).unwrap();

        let (o_interpreter, _, o_action) = deserialize_output(raw_output).unwrap();

        println!("o_interpreter.bytecode: {:?}", o_interpreter.bytecode);
        println!("o_interpreter.extend: {:?}", o_interpreter.extend);
        println!("o_interpreter.input: {:?}", o_interpreter.input);
        println!("o_interpreter.memory: {:?}", o_interpreter.memory);
        println!("o_interpreter.return_data: {:?}", o_interpreter.return_data);
        println!("o_interpreter.stack: {:?}", o_interpreter.stack);
        println!("o_interpreter.sub_routine: {:?}", o_interpreter.sub_routine);

        interpreter.bytecode = o_interpreter.bytecode;
        interpreter.control = o_interpreter.control;
        interpreter.extend = o_interpreter.extend;
        interpreter.input = o_interpreter.input;
        interpreter.memory = o_interpreter.memory;
        interpreter.return_data = o_interpreter.return_data;
        interpreter.stack = o_interpreter.stack;
        interpreter.sub_routine = o_interpreter.sub_routine;
        interpreter.runtime_flag = o_interpreter.runtime_flag;

        o_action
    }

    fn ctx_precompiles(&mut self) -> (&mut Self::Context, &mut Self::Precompiles) {
        self.0.ctx_precompiles()
    }
}

impl<CTX: ContextTr, INSP> InspectorEvmTr for HybridEvm<CTX, INSP>
where
    CTX: ContextSetters<Journal: JournalExt>,
    INSP: Inspector<CTX, EthInterpreter>,
{
    type Inspector = INSP;

    fn inspector(&mut self) -> &mut Self::Inspector {
        self.0.inspector()
    }

    fn ctx_inspector(&mut self) -> (&mut Self::Context, &mut Self::Inspector) {
        self.0.ctx_inspector()
    }

    fn run_inspect_interpreter(
        &mut self,
        interpreter: &mut Interpreter<
            <Self::Instructions as InstructionProvider>::InterpreterTypes,
        >,
    ) -> <<Self::Instructions as InstructionProvider>::InterpreterTypes as InterpreterTypes>::Output
    {
        let context = &mut self.0.data.ctx;
        let instructions = &mut self.0.instruction;
        let inspector = &mut self.0.data.inspector;

        inspect_instructions(
            context,
            interpreter,
            inspector,
            instructions.instruction_table(),
        )
    }
}

impl<CTX, INSP> HybridEvm<CTX, INSP> {
    /// Consumed self and returns new Evm type with given Inspector.
    pub fn with_inspector<OINSP>(self, inspector: OINSP) -> HybridEvm<CTX, OINSP> {
        HybridEvm(Evm {
            data: EvmData {
                ctx: self.0.data.ctx,
                inspector,
            },
            instruction: self.0.instruction,
            precompiles: self.0.precompiles,
        })
    }

    /// Consumes self and returns inner Inspector.
    pub fn into_inspector(self) -> INSP {
        self.0.data.inspector
    }
}
