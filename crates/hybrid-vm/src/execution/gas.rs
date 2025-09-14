// Standard EVM operation costs
pub const SLOAD_COLD: u64 = 2100;
pub const SLOAD_WARM: u64 = 100;
pub const SSTORE_COLD: u64 = 2200;
pub const SSTORE_WARM: u64 = 100;

// Call-related costs
pub const CALL_EMPTY_ACCOUNT: u64 = 25000;
pub const CALL_NEW_ACCOUNT: u64 = 2600;
pub const CALL_VALUE: u64 = 9000;
pub const CALL_BASE: u64 = 100;

// Create-related costs
pub const CREATE_BASE: u64 = 32000;

// Macro to handle gas accounting for syscalls.
// Returns OutOfGas InterpreterResult if gas limit is exceeded.
#[macro_export]
macro_rules! syscall_gas {
    ($interpreter:expr, $gas_cost:expr $(,)?) => {{
        let gas_cost = $gas_cost;

        if !$interpreter.control.gas.record_cost(gas_cost) {
            return Ok(InterpreterAction::Return {
                result: InterpreterResult {
                    result: InstructionResult::OutOfGas,
                    output: Bytes::new(),
                    gas: $interpreter.control.gas,
                },
            });
        }
    }};
}
