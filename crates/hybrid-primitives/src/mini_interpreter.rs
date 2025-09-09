use reth::revm::{
    context::{BlockEnv, CfgEnv, JournalTr, TxEnv},
    context_interface::context::ContextError,
    db::EmptyDB,
    interpreter::{Interpreter, InterpreterAction},
    Context, Database, Journal,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct InputRaw {
    pub context: MiniContext,
    pub interpreter: Interpreter,
}

pub struct Input {
    pub context: Context,
    pub interpreter: Interpreter,
}

#[derive(Serialize, Deserialize)]
pub struct OutputRaw {
    context: MiniContext,
    interpreter: Interpreter,
    out: InterpreterAction,
}

pub struct Output {
    pub context: Context,
    pub interpreter: Interpreter,
    pub out: InterpreterAction,
}

#[derive(Serialize, Deserialize)]
pub struct MiniContext<
    BLOCK = BlockEnv,
    TX = TxEnv,
    CFG = CfgEnv,
    DB: Database = EmptyDB,
    JOURNAL: JournalTr<Database = DB> = Journal<DB>,
    CHAIN = (),
> {
    /// Block information.
    pub block: BLOCK,
    /// Transaction information.
    pub tx: TX,
    /// Configurations.
    pub cfg: CFG,
    /// EVM State with journaling support and database.
    pub journaled_state: JOURNAL,
    /// Inner context.
    pub chain: CHAIN,
    #[serde(skip, default = "default_result::<DB>")]
    /// Error that happened during execution.
    pub error: Result<(), ContextError<DB::Error>>,
}

fn default_result<DB: Database>() -> Result<(), ContextError<DB::Error>> {
    Ok(())
}

impl InputRaw {
    pub fn new(context: MiniContext, interpreter: Interpreter) -> Self {
        Self {
            context,
            interpreter,
        }
    }

    pub fn into_input(self) -> Input {
        let context = Context {
            block: self.context.block,
            tx: self.context.tx,
            cfg: self.context.cfg,
            journaled_state: self.context.journaled_state,
            chain: self.context.chain,
            error: self.context.error,
        };

        Input {
            context,
            interpreter: self.interpreter,
        }
    }

    pub fn from_refined(context: Context, interpreter: Interpreter) -> Self {
        let context_out = MiniContext {
            block: context.block,
            tx: context.tx,
            cfg: context.cfg,
            journaled_state: context.journaled_state,
            chain: context.chain,
            error: context.error,
        };

        Self {
            context: context_out,
            interpreter,
        }
    }
}

impl OutputRaw {
    pub fn new(context: MiniContext, interpreter: Interpreter, out: InterpreterAction) -> Self {
        Self {
            context,
            interpreter,
            out,
        }
    }

    pub fn into_output(self) -> Output {
        let context = Context {
            block: self.context.block,
            tx: self.context.tx,
            cfg: self.context.cfg,
            journaled_state: self.context.journaled_state,
            chain: self.context.chain,
            error: self.context.error,
        };

        Output {
            context,
            interpreter: self.interpreter,
            out: self.out,
        }
    }

    pub fn from_refined(
        context: Context,
        interpreter: Interpreter,
        out: InterpreterAction,
    ) -> Self {
        let context_out = MiniContext {
            block: context.block,
            tx: context.tx,
            cfg: context.cfg,
            journaled_state: context.journaled_state,
            chain: context.chain,
            error: context.error,
        };

        Self {
            context: context_out,
            interpreter,
            out,
        }
    }
}

pub fn serialize_input(input: Input) -> Result<Vec<u8>, serde_json::Error> {
    let input = InputRaw::from_refined(input.context, input.interpreter);
    let serialized = serde_json::to_vec(&input)?;

    Ok(serialized)
}

pub fn deserialize_input(serialized: &[u8]) -> Result<Input, serde_json::Error> {
    let input: InputRaw = serde_json::from_slice(serialized)?;

    let context = Context {
        block: input.context.block,
        tx: input.context.tx,
        cfg: input.context.cfg,
        journaled_state: input.context.journaled_state,
        chain: input.context.chain,
        error: input.context.error,
    };

    let interpreter = input.interpreter;

    Ok(Input {
        context,
        interpreter,
    })
}

pub fn serialize_output(output: Output) -> Result<Vec<u8>, serde_json::Error> {
    let output = OutputRaw::from_refined(output.context, output.interpreter, output.out);
    let serialized = serde_json::to_vec(&output)?;

    Ok(serialized)
}

pub fn deserialize_output(serialized: &[u8]) -> Result<Output, serde_json::Error> {
    let output: OutputRaw = serde_json::from_slice(serialized)?;

    let context = Context {
        block: output.context.block,
        tx: output.context.tx,
        cfg: output.context.cfg,
        journaled_state: output.context.journaled_state,
        chain: output.context.chain,
        error: output.context.error,
    };

    let interpreter = output.interpreter;
    let out = output.out;

    Ok(Output {
        context,
        interpreter,
        out,
    })
}
