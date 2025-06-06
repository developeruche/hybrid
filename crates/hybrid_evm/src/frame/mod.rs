//! This file holds modifications to the EVM frame to accomdate the Hybrid logic.
use std::{cell::RefCell, rc::Rc};

use crate::riscv_execute::run_riscv_interpreter;
use reth::revm::{
    bytecode::EOF_MAGIC_BYTES, context::{
        result::{EVMError, FromStringError, InvalidTransaction}, Cfg, ContextTr, JournalOutput, JournalTr
    }, handler::{
        instructions::InstructionProvider, ContextTrDbError, EthFrame, EvmTr, Frame, FrameData, FrameInitOrResult, FrameOrResult, FrameResult, ItemOrResult, PrecompileProvider,
    }, interpreter::{interpreter::{EthInterpreter, ExtBytecode}, CallInputs, CallOutcome, CallValue, FrameInput, Gas, InputsImpl, InstructionResult, Interpreter, InterpreterResult, SharedMemory}, primitives::{Address, Bytes, CALL_STACK_LIMIT}, state::Bytecode, Database
};

pub struct HybridFrame<EVM, ERROR> 
where
    EVM: EvmTr<
        Precompiles: PrecompileProvider<EVM::Context, Output = InterpreterResult>,
        Instructions: InstructionProvider<
            Context = EVM::Context,
            InterpreterTypes = EthInterpreter,
        >,
    >,
{
    pub eth_frame: EthFrame<
        EVM,
        EVMError<<<EVM::Context as ContextTr>::Db as Database>::Error, InvalidTransaction>,
        <EVM::Instructions as InstructionProvider>::InterpreterTypes,
    >,
    created_address: Option<Address>,
    phantom: core::marker::PhantomData<(EVM, ERROR)>,
}



impl<EVM, ERROR> Frame for HybridFrame<EVM, ERROR>
where
    EVM: EvmTr<
        Precompiles: PrecompileProvider<EVM::Context, Output = InterpreterResult>,
        Instructions: InstructionProvider<
            Context = EVM::Context,
            InterpreterTypes = EthInterpreter,
        >,
    >,
    ERROR: From<ContextTrDbError<EVM::Context>> + FromStringError,
{
    type Evm = EVM;
    type FrameInit = FrameInput;
    type FrameResult = FrameResult;
    type Error = ERROR;

    fn init_first(
        evm: &mut Self::Evm,
        frame_input: Self::FrameInit,
    ) -> Result<FrameOrResult<Self>, Self::Error> {
        EthFrame::init_first(evm, frame_input)
    }

    fn init(
        &self,
        evm: &mut Self::Evm,
        frame_input: Self::FrameInit,
    ) -> Result<FrameOrResult<Self>, Self::Error> {
        self.init(evm, frame_input)
    }

    fn run(&mut self, context: &mut Self::Evm) -> Result<FrameInitOrResult<Self>, Self::Error> {
        let next_action = context.run_interpreter(&mut self.interpreter);
        self.process_next_action(context, next_action)
    }

    fn return_result(
        &mut self,
        context: &mut Self::Evm,
        result: Self::FrameResult,
    ) -> Result<(), Self::Error> {
        self.return_result(context, result)
    }
}




impl<EVM, ERROR> HybridFrame<EVM, ERROR>
where
    EVM: EvmTr<
        Precompiles: PrecompileProvider<EVM::Context, Output = InterpreterResult>,
        Instructions: InstructionProvider<
            Context = EVM::Context,
            InterpreterTypes = EthInterpreter,
        >,
    >,
    ERROR: From<ContextTrDbError<EVM::Context>>,
    ERROR: FromStringError,
{
    /// Make call frame
    #[inline]
    pub fn make_call_frame(
        evm: &mut EVM,
        depth: usize,
        memory: Rc<RefCell<SharedMemory>>,
        inputs: Box<CallInputs>,
    ) -> Result<ItemOrResult<Self, FrameResult>, ERROR> {
        let gas = Gas::new(inputs.gas_limit);

        let (context, precompiles) = evm.ctx_precompiles();

        let return_result = |instruction_result: InstructionResult| {
            Ok(ItemOrResult::Result(FrameResult::Call(CallOutcome {
                result: InterpreterResult {
                    result: instruction_result,
                    gas,
                    output: Bytes::new(),
                },
                memory_offset: inputs.return_memory_offset.clone(),
            })))
        };

        // Check depth
        if depth > CALL_STACK_LIMIT as usize {
            return return_result(InstructionResult::CallTooDeep);
        }

        // Make account warm and loaded
        let _ = context
            .journal()
            .load_account_delegated(inputs.bytecode_address)?;

        // Create subroutine checkpoint
        let checkpoint = context.journal().checkpoint();

        // Touch address. For "EIP-158 State Clear", this will erase empty accounts.
        if let CallValue::Transfer(value) = inputs.value {
            // Transfer value from caller to called account
            // Target will get touched even if balance transferred is zero.
            if let Some(i) =
                context
                    .journal()
                    .transfer(inputs.caller, inputs.target_address, value)?
            {
                context.journal().checkpoint_revert(checkpoint);
                return return_result(i.into());
            }
        }

        let interpreter_input = InputsImpl {
            target_address: inputs.target_address,
            caller_address: inputs.caller,
            input: inputs.input.clone(),
            call_value: inputs.value.get(),
        };
        let is_static = inputs.is_static;
        let gas_limit = inputs.gas_limit;

        let is_ext_delegate_call = inputs.scheme.is_ext_delegate_call();
        if !is_ext_delegate_call {
            if let Some(result) = precompiles
                .run(
                    context,
                    &inputs.bytecode_address,
                    &interpreter_input,
                    is_static,
                    gas_limit,
                )
                .map_err(ERROR::from_string)?
            {
                if result.result.is_ok() {
                    context.journal().checkpoint_commit();
                } else {
                    context.journal().checkpoint_revert(checkpoint);
                }
                return Ok(ItemOrResult::Result(FrameResult::Call(CallOutcome {
                    result,
                    memory_offset: inputs.return_memory_offset.clone(),
                })));
            }
        }

        let account = context
            .journal()
            .load_account_code(inputs.bytecode_address)?;

        let mut code_hash = account.info.code_hash();
        let mut bytecode = account.info.code.clone().unwrap_or_default();

        // ExtDelegateCall is not allowed to call non-EOF contracts.
        if is_ext_delegate_call && !bytecode.bytes_slice().starts_with(&EOF_MAGIC_BYTES) {
            context.journal().checkpoint_revert(checkpoint);
            return return_result(InstructionResult::InvalidExtDelegateCallTarget);
        }

        if bytecode.is_empty() {
            context.journal().checkpoint_commit();
            return return_result(InstructionResult::Stop);
        }

        if let Bytecode::Eip7702(eip7702_bytecode) = bytecode {
            let account = &context
                .journal()
                .load_account_code(eip7702_bytecode.delegated_address)?
                .info;
            bytecode = account.code.clone().unwrap_or_default();
            code_hash = account.code_hash();
        }

        // Create interpreter and executes call and push new CallStackFrame.
        let eth_frame = Self::new(
            FrameData::Call(CallFrame {
                return_memory_range: inputs.return_memory_offset.clone(),
            }),
            FrameInput::Call(inputs),
            depth,
            Interpreter::new(
                memory.clone(),
                ExtBytecode::new_with_hash(bytecode, code_hash),
                interpreter_input,
                is_static,
                false,
                context.cfg().spec().into(),
                gas_limit,
            ),
            checkpoint,
            memory,
        );
        
        Ok(ItemOrResult::Item())
    }

    /// Make create frame.
    #[inline]
    pub fn make_create_frame(
        evm: &mut EVM,
        depth: usize,
        memory: Rc<RefCell<SharedMemory>>,
        inputs: Box<CreateInputs>,
    ) -> Result<ItemOrResult<Self, FrameResult>, ERROR> {
        let context = evm.ctx();
        let spec = context.cfg().spec().into();
        let return_error = |e| {
            Ok(ItemOrResult::Result(FrameResult::Create(CreateOutcome {
                result: InterpreterResult {
                    result: e,
                    gas: Gas::new(inputs.gas_limit),
                    output: Bytes::new(),
                },
                address: None,
            })))
        };

        // Check depth
        if depth > CALL_STACK_LIMIT as usize {
            return return_error(InstructionResult::CallTooDeep);
        }

        // Prague EOF
        if spec.is_enabled_in(OSAKA) && inputs.init_code.starts_with(&EOF_MAGIC_BYTES) {
            return return_error(InstructionResult::CreateInitCodeStartingEF00);
        }

        // Fetch balance of caller.
        let caller_balance = context
            .journal()
            .load_account(inputs.caller)?
            .data
            .info
            .balance;

        // Check if caller has enough balance to send to the created contract.
        if caller_balance < inputs.value {
            return return_error(InstructionResult::OutOfFunds);
        }

        // Increase nonce of caller and check if it overflows
        let old_nonce;
        if let Some(nonce) = context.journal().inc_account_nonce(inputs.caller)? {
            old_nonce = nonce - 1;
        } else {
            return return_error(InstructionResult::Return);
        }

        // Create address
        let mut init_code_hash = B256::ZERO;
        let created_address = match inputs.scheme {
            CreateScheme::Create => inputs.caller.create(old_nonce),
            CreateScheme::Create2 { salt } => {
                init_code_hash = keccak256(&inputs.init_code);
                inputs.caller.create2(salt.to_be_bytes(), init_code_hash)
            }
        };

        // warm load account.
        context.journal().load_account(created_address)?;

        // Create account, transfer funds and make the journal checkpoint.
        let checkpoint = match context.journal().create_account_checkpoint(
            inputs.caller,
            created_address,
            inputs.value,
            spec,
        ) {
            Ok(checkpoint) => checkpoint,
            Err(e) => return return_error(e.into()),
        };

        let bytecode = ExtBytecode::new_with_hash(
            Bytecode::new_legacy(inputs.init_code.clone()),
            init_code_hash,
        );

        let interpreter_input = InputsImpl {
            target_address: created_address,
            caller_address: inputs.caller,
            input: Bytes::new(),
            call_value: inputs.value,
        };
        let gas_limit = inputs.gas_limit;
        Ok(ItemOrResult::Item(Self::new(
            FrameData::Create(CreateFrame { created_address }),
            FrameInput::Create(inputs),
            depth,
            Interpreter::new(
                memory.clone(),
                bytecode,
                interpreter_input,
                false,
                false,
                spec,
                gas_limit,
            ),
            checkpoint,
            memory,
        )))
    }

    /// Make create frame.
    #[inline]
    pub fn make_eofcreate_frame(
        evm: &mut EVM,
        depth: usize,
        memory: Rc<RefCell<SharedMemory>>,
        inputs: Box<EOFCreateInputs>,
    ) -> Result<ItemOrResult<Self, FrameResult>, ERROR> {
        let context = evm.ctx();
        let spec = context.cfg().spec().into();
        let return_error = |e| {
            Ok(ItemOrResult::Result(FrameResult::EOFCreate(
                CreateOutcome {
                    result: InterpreterResult {
                        result: e,
                        gas: Gas::new(inputs.gas_limit),
                        output: Bytes::new(),
                    },
                    address: None,
                },
            )))
        };

        let (input, initcode, created_address) = match &inputs.kind {
            EOFCreateKind::Opcode {
                initcode,
                input,
                created_address,
            } => (input.clone(), initcode.clone(), Some(*created_address)),
            EOFCreateKind::Tx { initdata } => {
                // Decode eof and init code.
                // TODO : Handle inc_nonce handling more gracefully.
                let Ok((eof, input)) = Eof::decode_dangling(initdata.clone()) else {
                    context.journal().inc_account_nonce(inputs.caller)?;
                    return return_error(InstructionResult::InvalidEOFInitCode);
                };

                if eof.validate().is_err() {
                    // TODO : (EOF) New error type.
                    context.journal().inc_account_nonce(inputs.caller)?;
                    return return_error(InstructionResult::InvalidEOFInitCode);
                }

                // Use nonce from tx to calculate address.
                let tx = context.tx();
                let create_address = tx.caller().create(tx.nonce());

                (input, eof, Some(create_address))
            }
        };

        // Check depth
        if depth > CALL_STACK_LIMIT as usize {
            return return_error(InstructionResult::CallTooDeep);
        }

        // Fetch balance of caller.
        let caller_balance = context
            .journal()
            .load_account(inputs.caller)?
            .map(|a| a.info.balance);

        // Check if caller has enough balance to send to the created contract.
        if caller_balance.data < inputs.value {
            return return_error(InstructionResult::OutOfFunds);
        }

        // Increase nonce of caller and check if it overflows
        let Some(nonce) = context.journal().inc_account_nonce(inputs.caller)? else {
            // Can't happen on mainnet.
            return return_error(InstructionResult::Return);
        };
        let old_nonce = nonce - 1;

        let created_address = created_address.unwrap_or_else(|| inputs.caller.create(old_nonce));

        // Load account so it needs to be marked as warm for access list.
        context.journal().load_account(created_address)?;

        // Create account, transfer funds and make the journal checkpoint.
        let checkpoint = match context.journal().create_account_checkpoint(
            inputs.caller,
            created_address,
            inputs.value,
            spec,
        ) {
            Ok(checkpoint) => checkpoint,
            Err(e) => return return_error(e.into()),
        };

        let interpreter_input = InputsImpl {
            target_address: created_address,
            caller_address: inputs.caller,
            input,
            call_value: inputs.value,
        };

        let gas_limit = inputs.gas_limit;
        Ok(ItemOrResult::Item(Self::new(
            FrameData::EOFCreate(EOFCreateFrame { created_address }),
            FrameInput::EOFCreate(inputs),
            depth,
            Interpreter::new(
                memory.clone(),
                ExtBytecode::new(Bytecode::Eof(Arc::new(initcode))),
                interpreter_input,
                false,
                true,
                spec,
                gas_limit,
            ),
            checkpoint,
            memory,
        )))
    }

    pub fn init_with_context(
        evm: &mut EVM,
        depth: usize,
        frame_init: FrameInput,
        memory: Rc<RefCell<SharedMemory>>,
    ) -> Result<ItemOrResult<Self, FrameResult>, ERROR> {
        match frame_init {
            FrameInput::Call(inputs) => Self::make_call_frame(evm, depth, memory, inputs),
            FrameInput::Create(inputs) => Self::make_create_frame(evm, depth, memory, inputs),
            FrameInput::EOFCreate(inputs) => Self::make_eofcreate_frame(evm, depth, memory, inputs),
        }
    }
}


impl<EVM, ERROR> HybridFrame<EVM, ERROR>
where
    EVM: EvmTr<
        Context: ContextTr,
        Precompiles: PrecompileProvider<EVM::Context, Output = InterpreterResult>,
        Instructions: InstructionProvider<
            Context = EVM::Context,
            InterpreterTypes = EthInterpreter,
        >,
    >,
    ERROR: From<ContextTrDbError<EVM::Context>> + FromStringError,
{
    pub fn init_first(
        evm: &mut EVM,
        frame_input: FrameInput,
    ) -> Result<ItemOrResult<Self, FrameResult>, ERROR> {
        let memory = Rc::new(RefCell::new(SharedMemory::new()));
        memory.borrow_mut().new_context();
        Self::init_with_context(evm, 0, frame_input, memory)
    }

    fn init(
        &self,
        evm: &mut EVM,
        frame_init: FrameInput,
    ) -> Result<ItemOrResult<Self, FrameResult>, ERROR> {
        self.memory.borrow_mut().new_context();
        Self::init_with_context(evm, self.depth + 1, frame_init, self.memory.clone())
    }

    pub fn process_next_action(
        &mut self,
        evm: &mut EVM,
        next_action: InterpreterAction,
    ) -> Result<FrameInitOrResult<Self>, ERROR> {
        let context = evm.ctx();
        let spec = context.cfg().spec().into();

        // Run interpreter

        let mut interpreter_result = match next_action {
            InterpreterAction::NewFrame(new_frame) => return Ok(ItemOrResult::Item(new_frame)),
            InterpreterAction::Return { result } => result,
            InterpreterAction::None => unreachable!("InterpreterAction::None is not expected"),
        };

        // Handle return from frame
        let result = match &self.data {
            FrameData::Call(frame) => {
                // return_call
                // Revert changes or not.
                if interpreter_result.result.is_ok() {
                    context.journal().checkpoint_commit();
                } else {
                    context.journal().checkpoint_revert(self.checkpoint);
                }
                ItemOrResult::Result(FrameResult::Call(CallOutcome::new(
                    interpreter_result,
                    frame.return_memory_range.clone(),
                )))
            }
            FrameData::Create(frame) => {
                let max_code_size = context.cfg().max_code_size();
                return_create(
                    context.journal(),
                    self.checkpoint,
                    &mut interpreter_result,
                    frame.created_address,
                    max_code_size,
                    spec,
                );

                ItemOrResult::Result(FrameResult::Create(CreateOutcome::new(
                    interpreter_result,
                    Some(frame.created_address),
                )))
            }
            FrameData::EOFCreate(frame) => {
                let max_code_size = context.cfg().max_code_size();
                return_eofcreate(
                    context.journal(),
                    self.checkpoint,
                    &mut interpreter_result,
                    frame.created_address,
                    max_code_size,
                );

                ItemOrResult::Result(FrameResult::EOFCreate(CreateOutcome::new(
                    interpreter_result,
                    Some(frame.created_address),
                )))
            }
        };

        Ok(result)
    }

    fn return_result(&mut self, evm: &mut EVM, result: FrameResult) -> Result<(), ERROR> {
        self.memory.borrow_mut().free_context();
        match core::mem::replace(evm.ctx().error(), Ok(())) {
            Err(ContextError::Db(e)) => return Err(e.into()),
            Err(ContextError::Custom(e)) => return Err(ERROR::from_string(e)),
            Ok(_) => (),
        }

        // Insert result to the top frame.
        match result {
            FrameResult::Call(outcome) => {
                let out_gas = outcome.gas();
                let ins_result = *outcome.instruction_result();
                let returned_len = outcome.result.output.len();

                let interpreter = &mut self.interpreter;
                let mem_length = outcome.memory_length();
                let mem_start = outcome.memory_start();
                interpreter.return_data.set_buffer(outcome.result.output);

                let target_len = min(mem_length, returned_len);

                if ins_result == InstructionResult::FatalExternalError {
                    panic!("Fatal external error in insert_call_outcome");
                }

                let item = {
                    if interpreter.runtime_flag.is_eof() {
                        match ins_result {
                            return_ok!() => U256::ZERO,
                            return_revert!() => U256::from(1),
                            _ => U256::from(2),
                        }
                    } else if ins_result.is_ok() {
                        U256::from(1)
                    } else {
                        U256::ZERO
                    }
                };
                // Safe to push without stack limit check
                let _ = interpreter.stack.push(item);

                // Return unspend gas.
                if ins_result.is_ok_or_revert() {
                    interpreter
                        .control
                        .gas_mut()
                        .erase_cost(out_gas.remaining());
                    self.memory
                        .borrow_mut()
                        .set(mem_start, &interpreter.return_data.buffer()[..target_len]);
                }

                if ins_result.is_ok() {
                    interpreter
                        .control
                        .gas_mut()
                        .record_refund(out_gas.refunded());
                }
            }
            FrameResult::Create(outcome) => {
                let instruction_result = *outcome.instruction_result();
                let interpreter = &mut self.interpreter;

                if instruction_result == InstructionResult::Revert {
                    // Save data to return data buffer if the create reverted
                    interpreter
                        .return_data
                        .set_buffer(outcome.output().to_owned());
                } else {
                    // Otherwise clear it. Note that RETURN opcode should abort.
                    interpreter.return_data.clear();
                };

                assert_ne!(
                    instruction_result,
                    InstructionResult::FatalExternalError,
                    "Fatal external error in insert_eofcreate_outcome"
                );

                let this_gas = interpreter.control.gas_mut();
                if instruction_result.is_ok_or_revert() {
                    this_gas.erase_cost(outcome.gas().remaining());
                }

                let stack_item = if instruction_result.is_ok() {
                    this_gas.record_refund(outcome.gas().refunded());
                    outcome.address.unwrap_or_default().into_word().into()
                } else {
                    U256::ZERO
                };

                // Safe to push without stack limit check
                let _ = interpreter.stack.push(stack_item);
            }
            FrameResult::EOFCreate(outcome) => {
                let instruction_result = *outcome.instruction_result();
                let interpreter = &mut self.interpreter;
                if instruction_result == InstructionResult::Revert {
                    // Save data to return data buffer if the create reverted
                    interpreter
                        .return_data
                        .set_buffer(outcome.output().to_owned());
                } else {
                    // Otherwise clear it. Note that RETURN opcode should abort.
                    interpreter.return_data.clear()
                };

                assert_ne!(
                    instruction_result,
                    InstructionResult::FatalExternalError,
                    "Fatal external error in insert_eofcreate_outcome"
                );

                let this_gas = interpreter.control.gas_mut();
                if instruction_result.is_ok_or_revert() {
                    this_gas.erase_cost(outcome.gas().remaining());
                }

                let stack_item = if instruction_result.is_ok() {
                    this_gas.record_refund(outcome.gas().refunded());
                    outcome.address.expect("EOF Address").into_word().into()
                } else {
                    U256::ZERO
                };

                // Safe to push without stack limit check
                let _ = interpreter.stack.push(stack_item);
            }
        }

        Ok(())
    }
}










pub fn hybrid_frame_call<EVM>(
    frame: &mut EthFrame<
        EVM,
        EVMError<<<EVM::Context as ContextTr>::Db as Database>::Error, InvalidTransaction>,
        <EVM::Instructions as InstructionProvider>::InterpreterTypes,
    >,
    evm: &mut EVM,
) -> Result<
    FrameInitOrResult<
        EthFrame<
            EVM,
            EVMError<<<EVM::Context as ContextTr>::Db as Database>::Error, InvalidTransaction>,
            <EVM::Instructions as InstructionProvider>::InterpreterTypes,
        >,
    >,
    EVMError<<<EVM::Context as ContextTr>::Db as Database>::Error, InvalidTransaction>,
>
where
    EVM: EvmTr<
            Context: ContextTr<Journal: JournalTr<FinalOutput = JournalOutput>>,
            Precompiles: PrecompileProvider<EVM::Context, Output = InterpreterResult>,
            Instructions: InstructionProvider<
                Context = EVM::Context,
                InterpreterTypes = EthInterpreter,
            >,
        >,
{
    let bytecode_clone = frame.interpreter.bytecode.clone();
    let split_result = bytecode_clone.bytecode().split_first();

    if split_result.is_some() && *split_result.unwrap().0 == 0xFF {
        let (_, bytecode) = split_result.unwrap();

        return run_riscv_interpreter::<
            EVM,
            EVMError<<<EVM::Context as ContextTr>::Db as Database>::Error, InvalidTransaction>,
        >(bytecode, frame, evm);
    } else {
        return Frame::run(frame, evm);
    }
}
