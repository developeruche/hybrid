use revm::{
    Inspector,
    context::{ContextSetters, ContextTr, Evm},
    handler::{
        EthPrecompiles, EvmTr,
        instructions::{EthInstructions, InstructionProvider},
    },
    inspector::{InspectorEvmTr, JournalExt, inspect_instructions},
    interpreter::{Interpreter, InterpreterTypes, interpreter::EthInterpreter},
};

/// HybridEvm variant of the EVM.
pub struct HybridEvm<CTX, INSP>(
    pub Evm<CTX, INSP, EthInstructions<EthInterpreter, CTX>, EthPrecompiles>,
);

impl<CTX, INSP> HybridEvm<CTX, INSP> {
    /// Consumed self and returns new Evm type with given Inspector.
    pub fn with_inspector<OINSP>(self, inspector: OINSP) -> Evm<CTX, OINSP, EthInstructions<EthInterpreter, CTX>, EthPrecompiles> {
        Evm {
            ctx: self.0.ctx,
            inspector,

            instruction: self.0.instruction,
            precompiles: self.0.precompiles,
        }
    }
    
    /// Consumes self and returns new Evm type with given Precompiles.
    pub fn with_precompiles<OP>(self, precompiles: OP) -> Evm<CTX, INSP, EthInstructions<EthInterpreter, CTX>, OP> {
        Evm {
            ctx: self.0.ctx,
            inspector: self.0.inspector,
            instruction: self.0.instruction,
            precompiles,
        }
    }
    
    /// Consumes self and returns inner Inspector.
    pub fn into_inspector(self) -> INSP {
        self.0.inspector
    }
}

impl<CTX: ContextTr, INSP> HybridEvm<CTX, INSP> {
    pub fn new(ctx: CTX, inspector: INSP) -> Self {
        Self(Evm {
            ctx,
            inspector,
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
        &mut self.0.ctx
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
        self.0.run_interpreter(interpreter)
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
        let context = &mut self.0.ctx;
        let instructions = &mut self.0.instruction;
        let inspector = &mut self.0.inspector;

        inspect_instructions(
            context,
            interpreter,
            inspector,
            instructions.instruction_table(),
        )
    }
}
