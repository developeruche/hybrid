use reth::revm::{
    context::{
        Block, BlockEnv, Cfg, CfgEnv, ContextSetters, ContextTr, Evm, EvmData, Transaction, TxEnv,
    },
    handler::{
        instructions::{EthInstructions, InstructionProvider},
        EthPrecompiles, EvmTr,
    },
    inspector::{inspect_instructions, InspectorEvmTr, JournalExt},
    interpreter::{interpreter::EthInterpreter, Interpreter, InterpreterTypes},
    Inspector,
};

use crate::{
    execution::helper::dram_slice,
    mini_evm_coding::{deserialize_output, serialize_input},
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

        let emu_input = serialize_input(&interpreter, &block, &tx);

        #[cfg(test)]
        let mini_evm_bin: &[u8] = include_bytes!("../../../bins/mini-evm-interpreter/target/riscv64imac-unknown-none-elf/release/runtime");

        #[cfg(not(test))]
        let mini_evm_bin: &[u8] = include_bytes!("../mini-evm-interpreter");

        let mut emulator = match setup_from_mini_elf(mini_evm_bin, &emu_input) {
            Ok(emulator) => emulator,
            Err(err) => {
                // TODO:: handle this gracefully
                panic!("Error occurred setting up emulator: {}", err)
            }
        };

        let return_res = emulator.estart();

        match return_res {
            Ok(_) => (),
            Err(err) => {
                // TODO: Here syscalls would be handled
                println!("Emulator Error Occured: {:?}", err);
            }
        }

        let interpreter_output_size: u64 = emulator.cpu.xregs.read(31);

        let raw_output = dram_slice(&mut emulator, 0x8000_0000, interpreter_output_size).unwrap();

        let (_, _, _, o_out) = deserialize_output(raw_output);

        o_out
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

#[cfg(test)]
mod tests {
    use super::*;
    use rvemu::bus::DRAM_BASE;

    #[test]
    fn test_mini_interpreter_emulator() {
        let emu_input = &[
            142, 2, 0, 0, 0, 0, 0, 0, 166, 0, 0, 0, 0, 0, 0, 0, 152, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 25, 1, 0, 0, 0, 0, 0, 0, 96, 128, 128, 96, 64, 82, 52, 96, 19, 87, 96, 223, 144,
            129, 96, 25, 130, 57, 243, 91, 96, 0, 128, 253, 254, 96, 128, 128, 96, 64, 82, 96, 4,
            54, 16, 21, 96, 18, 87, 96, 0, 128, 253, 91, 96, 0, 53, 96, 224, 28, 144, 129, 99, 63,
            181, 193, 203, 20, 96, 146, 87, 129, 99, 131, 129, 245, 138, 20, 96, 121, 87, 80, 99,
            208, 157, 224, 138, 20, 96, 60, 87, 96, 0, 128, 253, 91, 52, 96, 116, 87, 96, 0, 54,
            96, 3, 25, 1, 18, 96, 116, 87, 96, 0, 84, 96, 0, 25, 129, 20, 96, 94, 87, 96, 1, 1, 96,
            0, 85, 0, 91, 99, 78, 72, 123, 113, 96, 224, 27, 96, 0, 82, 96, 17, 96, 4, 82, 96, 36,
            96, 0, 253, 91, 96, 0, 128, 253, 91, 52, 96, 116, 87, 96, 0, 54, 96, 3, 25, 1, 18, 96,
            116, 87, 96, 32, 144, 96, 0, 84, 129, 82, 243, 91, 52, 96, 116, 87, 96, 32, 54, 96, 3,
            25, 1, 18, 96, 116, 87, 96, 4, 53, 96, 0, 85, 0, 254, 162, 100, 105, 112, 102, 115, 88,
            34, 18, 32, 233, 120, 39, 8, 131, 183, 186, 237, 16, 129, 12, 64, 121, 201, 65, 81, 46,
            147, 167, 186, 28, 209, 16, 140, 120, 29, 75, 199, 56, 217, 9, 5, 100, 115, 111, 108,
            99, 67, 0, 8, 26, 0, 51, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 248, 0, 0, 0, 0, 0, 0, 0, 19, 0, 0, 0, 0, 0, 0, 0,
            98, 105, 116, 118, 101, 99, 58, 58, 111, 114, 100, 101, 114, 58, 58, 76, 115, 98, 48,
            8, 0, 25, 1, 0, 0, 0, 0, 0, 0, 36, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 8, 0, 0, 0, 0,
            32, 0, 0, 0, 128, 0, 0, 32, 4, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 1, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 20, 0, 0, 0, 0, 0, 0, 0, 95, 189, 178, 49, 86, 120, 175, 236,
            179, 103, 240, 50, 217, 63, 100, 47, 100, 24, 10, 163, 20, 0, 0, 0, 0, 0, 0, 0, 243,
            159, 214, 229, 26, 173, 136, 246, 244, 206, 106, 184, 130, 114, 121, 207, 255, 185, 34,
            102, 0, 0, 0, 0, 0, 0, 0, 0, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 64, 34, 255, 255, 255, 135, 243, 0, 64,
            34, 255, 255, 255, 135, 243, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 18, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 20, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 12, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 136, 243, 0, 192, 112, 39, 52, 0, 0, 0, 0, 32, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 1, 32, 0, 0, 0, 0, 0, 0, 0, 24, 237, 82, 51, 255, 14, 71, 106, 163, 81, 42, 166,
            92, 183, 219, 159, 192, 230, 159, 203, 190, 118, 182, 87, 54, 122, 250, 58, 162, 137,
            116, 14, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            20, 0, 0, 0, 0, 0, 0, 0, 243, 159, 214, 229, 26, 173, 136, 246, 244, 206, 106, 184,
            130, 114, 121, 207, 255, 185, 34, 102, 0, 0, 0, 0, 0, 136, 243, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 248, 0, 0, 0, 0, 0, 0,
            0, 96, 128, 128, 96, 64, 82, 52, 96, 19, 87, 96, 223, 144, 129, 96, 25, 130, 57, 243,
            91, 96, 0, 128, 253, 254, 96, 128, 128, 96, 64, 82, 96, 4, 54, 16, 21, 96, 18, 87, 96,
            0, 128, 253, 91, 96, 0, 53, 96, 224, 28, 144, 129, 99, 63, 181, 193, 203, 20, 96, 146,
            87, 129, 99, 131, 129, 245, 138, 20, 96, 121, 87, 80, 99, 208, 157, 224, 138, 20, 96,
            60, 87, 96, 0, 128, 253, 91, 52, 96, 116, 87, 96, 0, 54, 96, 3, 25, 1, 18, 96, 116, 87,
            96, 0, 84, 96, 0, 25, 129, 20, 96, 94, 87, 96, 1, 1, 96, 0, 85, 0, 91, 99, 78, 72, 123,
            113, 96, 224, 27, 96, 0, 82, 96, 17, 96, 4, 82, 96, 36, 96, 0, 253, 91, 96, 0, 128,
            253, 91, 52, 96, 116, 87, 96, 0, 54, 96, 3, 25, 1, 18, 96, 116, 87, 96, 32, 144, 96, 0,
            84, 129, 82, 243, 91, 52, 96, 116, 87, 96, 32, 54, 96, 3, 25, 1, 18, 96, 116, 87, 96,
            4, 53, 96, 0, 85, 0, 254, 162, 100, 105, 112, 102, 115, 88, 34, 18, 32, 233, 120, 39,
            8, 131, 183, 186, 237, 16, 129, 12, 64, 121, 201, 65, 81, 46, 147, 167, 186, 28, 209,
            16, 140, 120, 29, 75, 199, 56, 217, 9, 5, 100, 115, 111, 108, 99, 67, 0, 8, 26, 0, 51,
            0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];

        let mini_evm_bin: &[u8] = include_bytes!("../../../bins/mini-evm-interpreter/target/riscv64imac-unknown-none-elf/release/runtime");

        let mut emulator = match setup_from_mini_elf(mini_evm_bin, emu_input) {
            Ok(emulator) => emulator,
            Err(_err) => {
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

        let debug_addr = DRAM_BASE + (1024 * 1024 * 1024) - 2000;
        let debug_output = dram_slice(&mut emulator, debug_addr, 13).unwrap();
        println!("Out Debug:: -> {:?}", std::str::from_utf8(debug_output));
    }
}
