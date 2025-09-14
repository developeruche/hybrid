//! EVM Instruction Table for Mini EVM Interpreter
//!
//! This module defines a custom instruction table that maps EVM opcodes to their corresponding
//! implementation functions. The mini-EVM supports a comprehensive subset of Ethereum Virtual
//! Machine opcodes, organized by functional categories.
//!
//! ## Supported Instruction Categories
//!
//! - **Arithmetic**: Basic math operations (ADD, SUB, MUL, DIV, etc.)
//! - **Bitwise**: Logical operations (AND, OR, XOR, NOT, shifts, etc.)
//! - **Comparison**: Comparison operations (LT, GT, EQ, etc.)
//! - **Stack**: Stack manipulation (PUSH, POP, DUP, SWAP)
//! - **Memory**: Memory operations (MLOAD, MSTORE, MCOPY)
//! - **Storage**: Persistent storage (SLOAD, SSTORE, TLOAD, TSTORE)
//! - **Control Flow**: Execution control (JUMP, JUMPI, CALL, RETURN)
//! - **Environment**: Context information (CALLER, ORIGIN, TIMESTAMP, etc.)
//! - **System**: System operations (KECCAK256, CREATE, SELFDESTRUCT)
//!
//! ## Architecture
//!
//! The instruction table is implemented as a constant function that returns a 256-element array,
//! where each index corresponds to an EVM opcode (0x00-0xFF). Unsupported opcodes default to
//! the `unknown` instruction handler.
//!
//! ## Host Dependencies
//!
//! Many instructions require host environment interaction for:
//! - Account balance queries
//! - Storage access
//! - Contract creation and calls
//! - Event logging
//! - Block information retrieval
//!
//! These operations are marked with comments indicating their host dependency level.

use ext_revm::{
    bytecode::opcode::*,
    interpreter::{
        instructions::{
            arithmetic, bitwise, block_info, contract, control, data, host, memory, stack, system,
            tx_info,
        },
        Host, Instruction, InterpreterTypes,
    },
};

/// Creates a comprehensive EVM instruction table for the mini-EVM interpreter.
///
/// This function builds a complete opcode mapping table that defines how each EVM instruction
/// should be handled. The table covers all major EVM instruction categories and provides
/// fallback handling for unsupported opcodes.
///
/// ## Generic Parameters
///
/// * `WIRE` - The interpreter types configuration that defines the execution environment
/// * `H` - The host trait implementation that provides external environment access
///
/// ## Returns
///
/// Returns a 256-element array where each index corresponds to an EVM opcode (0x00-0xFF).
/// Each element is an `Instruction` function pointer that implements the opcode's behavior.
///
/// ## Instruction Categories
///
/// ### Core Operations (0x00-0x0F)
/// - **STOP**: Halts execution
/// - **Arithmetic**: ADD, MUL, SUB, DIV, SDIV, MOD, SMOD, ADDMOD, MULMOD, EXP, SIGNEXTEND
///
/// ### Comparison & Bitwise (0x10-0x1F)
/// - **Comparison**: LT, GT, SLT, SGT, EQ, ISZERO
/// - **Bitwise**: AND, OR, XOR, NOT, BYTE, SHL, SHR, SAR
/// - **Hashing**: KECCAK256
///
/// ### Environment Information (0x30-0x4F)
/// - **Address Info**: ADDRESS, BALANCE, ORIGIN, CALLER, CALLVALUE
/// - **Calldata**: CALLDATALOAD, CALLDATASIZE, CALLDATACOPY
/// - **Code**: CODESIZE, CODECOPY, GASPRICE
/// - **External**: EXTCODESIZE, EXTCODECOPY, RETURNDATASIZE, RETURNDATACOPY, EXTCODEHASH
/// - **Block Info**: BLOCKHASH, COINBASE, TIMESTAMP, NUMBER, DIFFICULTY, GASLIMIT, CHAINID, BASEFEE
/// - **Advanced**: BLOBHASH, BLOBBASEFEE, SELFBALANCE
///
/// ### Stack & Memory Operations (0x50-0x5F)
/// - **Stack**: POP
/// - **Memory**: MLOAD, MSTORE, MSTORE8, MSIZE, MCOPY
/// - **Storage**: SLOAD, SSTORE, TLOAD, TSTORE
/// - **Control**: JUMP, JUMPI, PC, GAS, JUMPDEST
///
/// ### Push Operations (0x60-0x7F)
/// - **PUSH0-PUSH32**: Push 0 to 32 bytes onto the stack
///
/// ### Duplicate Operations (0x80-0x8F)
/// - **DUP1-DUP16**: Duplicate stack items at various depths
///
/// ### Swap Operations (0x90-0x9F)
/// - **SWAP1-SWAP16**: Swap stack items at various depths
///
/// ### Logging (0xA0-0xA4)
/// - **LOG0-LOG4**: Emit events with 0 to 4 indexed topics
///
/// ### EOF (Ethereum Object Format) Operations (0xD0-0xEF)
/// - **Data Operations**: DATALOAD, DATALOADN, DATASIZE, DATACOPY
/// - **Control Flow**: RJUMP, RJUMPI, RJUMPV, CALLF, RETF, JUMPF
/// - **Stack**: DUPN, SWAPN, EXCHANGE
/// - **Contracts**: EOFCREATE, RETURNCONTRACT
///
/// ### System Operations (0xF0-0xFF)
/// - **Contract Operations**: CREATE, CALL, CALLCODE, RETURN, DELEGATECALL, CREATE2
/// - **Advanced Calls**: RETURNDATALOAD, EXTCALL, EXTDELEGATECALL, STATICCALL, EXTSTATICCALL
/// - **Termination**: REVERT, INVALID, SELFDESTRUCT
///
/// ## Host Interaction Points
///
/// Several instructions require host environment interaction (marked with numbered comments):
/// 1. BALANCE - Query account balance
/// 2. EXTCODESIZE - Get external contract code size
/// 3. EXTCODECOPY - Copy external contract code
/// 4. EXTCODEHASH - Get external contract code hash
/// 5. BLOCKHASH - Get block hash for given block number
/// 6. SELFBALANCE - Get current contract's balance
/// 7. SLOAD - Load from contract storage
/// 8. SSTORE - Store to contract storage
/// 9. TLOAD - Load from transient storage
/// 10. TSTORE - Store to transient storage
/// 11-15. LOG0-LOG4 - Emit events
/// 15-22. Various CALL operations and contract interactions
///
/// ## Example Usage
///
/// ```rust,no_run
/// use ext_revm::interpreter::{Interpreter, InterpreterTypes};
/// use ext_revm::context::Context;
///
/// let table = mini_instruction_table::<DefaultInterpreterTypes, MyHost>();
/// let result = interpreter.run_plain(&table, &mut context);
/// ```
pub const fn mini_instruction_table<WIRE: InterpreterTypes, H: Host + ?Sized>(
) -> [Instruction<WIRE, H>; 256] {
    // Initialize all opcodes to the unknown instruction handler
    // This provides safe fallback behavior for unsupported opcodes
    let mut table = [control::unknown as Instruction<WIRE, H>; 256];

    // === Core Control and Arithmetic Operations (0x00-0x0B) ===
    table[STOP as usize] = control::stop; // 0x00: Halt execution
    table[ADD as usize] = arithmetic::add; // 0x01: Addition
    table[MUL as usize] = arithmetic::mul; // 0x02: Multiplication
    table[SUB as usize] = arithmetic::sub; // 0x03: Subtraction
    table[DIV as usize] = arithmetic::div; // 0x04: Integer division
    table[SDIV as usize] = arithmetic::sdiv; // 0x05: Signed integer division
    table[MOD as usize] = arithmetic::rem; // 0x06: Modulo remainder
    table[SMOD as usize] = arithmetic::smod; // 0x07: Signed modulo
    table[ADDMOD as usize] = arithmetic::addmod; // 0x08: Modular addition
    table[MULMOD as usize] = arithmetic::mulmod; // 0x09: Modular multiplication
    table[EXP as usize] = arithmetic::exp; // 0x0A: Exponentiation
    table[SIGNEXTEND as usize] = arithmetic::signextend; // 0x0B: Sign extension

    // === Comparison and Bitwise Operations (0x10-0x1D) ===
    table[LT as usize] = bitwise::lt; // 0x10: Less-than comparison
    table[GT as usize] = bitwise::gt; // 0x11: Greater-than comparison
    table[SLT as usize] = bitwise::slt; // 0x12: Signed less-than
    table[SGT as usize] = bitwise::sgt; // 0x13: Signed greater-than
    table[EQ as usize] = bitwise::eq; // 0x14: Equality comparison
    table[ISZERO as usize] = bitwise::iszero; // 0x15: Is-zero check
    table[AND as usize] = bitwise::bitand; // 0x16: Bitwise AND
    table[OR as usize] = bitwise::bitor; // 0x17: Bitwise OR
    table[XOR as usize] = bitwise::bitxor; // 0x18: Bitwise XOR
    table[NOT as usize] = bitwise::not; // 0x19: Bitwise NOT
    table[BYTE as usize] = bitwise::byte; // 0x1A: Byte extraction
    table[SHL as usize] = bitwise::shl; // 0x1B: Shift left
    table[SHR as usize] = bitwise::shr; // 0x1C: Logical shift right
    table[SAR as usize] = bitwise::sar; // 0x1D: Arithmetic shift right

    // === Cryptographic Operations (0x20) ===
    table[KECCAK256 as usize] = system::keccak256; // 0x20: Keccak-256 hash

    // === Environment Information (0x30-0x39) ===
    table[ADDRESS as usize] = system::address; // 0x30: Get executing contract address
    table[BALANCE as usize] = host::balance; // 0x31: Get account balance [HOST]
    table[ORIGIN as usize] = tx_info::origin; // 0x32: Get transaction origin
    table[CALLER as usize] = system::caller; // 0x33: Get message caller
    table[CALLVALUE as usize] = system::callvalue; // 0x34: Get sent value
    table[CALLDATALOAD as usize] = system::calldataload; // 0x35: Load word from calldata
    table[CALLDATASIZE as usize] = system::calldatasize; // 0x36: Get calldata size
    table[CALLDATACOPY as usize] = system::calldatacopy; // 0x37: Copy calldata to memory
    table[CODESIZE as usize] = system::codesize; // 0x38: Get code size
    table[CODECOPY as usize] = system::codecopy; // 0x39: Copy code to memory

    // === Extended Environment and Block Information (0x3A-0x4A) ===
    table[GASPRICE as usize] = tx_info::gasprice; // 0x3A: Get gas price
    table[EXTCODESIZE as usize] = host::extcodesize; // 0x3B: External code size [HOST]
    table[EXTCODECOPY as usize] = host::extcodecopy; // 0x3C: Copy external code [HOST]
    table[RETURNDATASIZE as usize] = system::returndatasize; // 0x3D: Return data size
    table[RETURNDATACOPY as usize] = system::returndatacopy; // 0x3E: Copy return data
    table[EXTCODEHASH as usize] = host::extcodehash; // 0x3F: External code hash [HOST]
    table[BLOCKHASH as usize] = host::blockhash; // 0x40: Get block hash [HOST]
    table[COINBASE as usize] = block_info::coinbase; // 0x41: Get coinbase address
    table[TIMESTAMP as usize] = block_info::timestamp; // 0x42: Get block timestamp
    table[NUMBER as usize] = block_info::block_number; // 0x43: Get block number
    table[DIFFICULTY as usize] = block_info::difficulty; // 0x44: Get block difficulty
    table[GASLIMIT as usize] = block_info::gaslimit; // 0x45: Get block gas limit
    table[CHAINID as usize] = block_info::chainid; // 0x46: Get chain identifier
    table[SELFBALANCE as usize] = host::selfbalance; // 0x47: Get own balance [HOST]
    table[BASEFEE as usize] = block_info::basefee; // 0x48: Get base fee
    table[BLOBHASH as usize] = tx_info::blob_hash; // 0x49: Get blob hash
    table[BLOBBASEFEE as usize] = block_info::blob_basefee; // 0x4A: Get blob base fee

    // === Stack, Memory, Storage and Control Operations (0x50-0x5E) ===
    table[POP as usize] = stack::pop; // 0x50: Remove top stack item
    table[MLOAD as usize] = memory::mload; // 0x51: Load word from memory
    table[MSTORE as usize] = memory::mstore; // 0x52: Store word to memory
    table[MSTORE8 as usize] = memory::mstore8; // 0x53: Store byte to memory
    table[SLOAD as usize] = host::sload; // 0x54: Load from storage [HOST]
    table[SSTORE as usize] = host::sstore; // 0x55: Store to storage [HOST]
    table[JUMP as usize] = control::jump; // 0x56: Unconditional jump
    table[JUMPI as usize] = control::jumpi; // 0x57: Conditional jump
    table[PC as usize] = control::pc; // 0x58: Get program counter
    table[MSIZE as usize] = memory::msize; // 0x59: Get memory size
    table[GAS as usize] = system::gas; // 0x5A: Get available gas
    table[JUMPDEST as usize] = control::jumpdest_or_nop; // 0x5B: Jump destination marker
    table[TLOAD as usize] = host::tload; // 0x5C: Load from transient storage [HOST]
    table[TSTORE as usize] = host::tstore; // 0x5D: Store to transient storage [HOST]
    table[MCOPY as usize] = memory::mcopy; // 0x5E: Copy memory region

    // === Push Operations (0x5F-0x7F) ===
    // Push 0 to 32 bytes onto the stack
    table[PUSH0 as usize] = stack::push0; // 0x5F: Push zero
    table[PUSH1 as usize] = stack::push::<1, _, _>; // 0x60: Push 1 byte
    table[PUSH2 as usize] = stack::push::<2, _, _>; // 0x61: Push 2 bytes
    table[PUSH3 as usize] = stack::push::<3, _, _>; // 0x62: Push 3 bytes
    table[PUSH4 as usize] = stack::push::<4, _, _>; // 0x63: Push 4 bytes
    table[PUSH5 as usize] = stack::push::<5, _, _>; // 0x64: Push 5 bytes
    table[PUSH6 as usize] = stack::push::<6, _, _>; // 0x65: Push 6 bytes
    table[PUSH7 as usize] = stack::push::<7, _, _>; // 0x66: Push 7 bytes
    table[PUSH8 as usize] = stack::push::<8, _, _>; // 0x67: Push 8 bytes
    table[PUSH9 as usize] = stack::push::<9, _, _>; // 0x68: Push 9 bytes
    table[PUSH10 as usize] = stack::push::<10, _, _>; // 0x69: Push 10 bytes
    table[PUSH11 as usize] = stack::push::<11, _, _>; // 0x6A: Push 11 bytes
    table[PUSH12 as usize] = stack::push::<12, _, _>; // 0x6B: Push 12 bytes
    table[PUSH13 as usize] = stack::push::<13, _, _>; // 0x6C: Push 13 bytes
    table[PUSH14 as usize] = stack::push::<14, _, _>; // 0x6D: Push 14 bytes
    table[PUSH15 as usize] = stack::push::<15, _, _>; // 0x6E: Push 15 bytes
    table[PUSH16 as usize] = stack::push::<16, _, _>; // 0x6F: Push 16 bytes
    table[PUSH17 as usize] = stack::push::<17, _, _>; // 0x70: Push 17 bytes
    table[PUSH18 as usize] = stack::push::<18, _, _>; // 0x71: Push 18 bytes
    table[PUSH19 as usize] = stack::push::<19, _, _>; // 0x72: Push 19 bytes
    table[PUSH20 as usize] = stack::push::<20, _, _>; // 0x73: Push 20 bytes
    table[PUSH21 as usize] = stack::push::<21, _, _>; // 0x74: Push 21 bytes
    table[PUSH22 as usize] = stack::push::<22, _, _>; // 0x75: Push 22 bytes
    table[PUSH23 as usize] = stack::push::<23, _, _>; // 0x76: Push 23 bytes
    table[PUSH24 as usize] = stack::push::<24, _, _>; // 0x77: Push 24 bytes
    table[PUSH25 as usize] = stack::push::<25, _, _>; // 0x78: Push 25 bytes
    table[PUSH26 as usize] = stack::push::<26, _, _>; // 0x79: Push 26 bytes
    table[PUSH27 as usize] = stack::push::<27, _, _>; // 0x7A: Push 27 bytes
    table[PUSH28 as usize] = stack::push::<28, _, _>; // 0x7B: Push 28 bytes
    table[PUSH29 as usize] = stack::push::<29, _, _>; // 0x7C: Push 29 bytes
    table[PUSH30 as usize] = stack::push::<30, _, _>; // 0x7D: Push 30 bytes
    table[PUSH31 as usize] = stack::push::<31, _, _>; // 0x7E: Push 31 bytes
    table[PUSH32 as usize] = stack::push::<32, _, _>; // 0x7F: Push 32 bytes

    // === Duplicate Operations (0x80-0x8F) ===
    // Duplicate stack items from various depths
    table[DUP1 as usize] = stack::dup::<1, _, _>; // 0x80: Duplicate 1st stack item
    table[DUP2 as usize] = stack::dup::<2, _, _>; // 0x81: Duplicate 2nd stack item
    table[DUP3 as usize] = stack::dup::<3, _, _>; // 0x82: Duplicate 3rd stack item
    table[DUP4 as usize] = stack::dup::<4, _, _>; // 0x83: Duplicate 4th stack item
    table[DUP5 as usize] = stack::dup::<5, _, _>; // 0x84: Duplicate 5th stack item
    table[DUP6 as usize] = stack::dup::<6, _, _>; // 0x85: Duplicate 6th stack item
    table[DUP7 as usize] = stack::dup::<7, _, _>; // 0x86: Duplicate 7th stack item
    table[DUP8 as usize] = stack::dup::<8, _, _>; // 0x87: Duplicate 8th stack item
    table[DUP9 as usize] = stack::dup::<9, _, _>; // 0x88: Duplicate 9th stack item
    table[DUP10 as usize] = stack::dup::<10, _, _>; // 0x89: Duplicate 10th stack item
    table[DUP11 as usize] = stack::dup::<11, _, _>; // 0x8A: Duplicate 11th stack item
    table[DUP12 as usize] = stack::dup::<12, _, _>; // 0x8B: Duplicate 12th stack item
    table[DUP13 as usize] = stack::dup::<13, _, _>; // 0x8C: Duplicate 13th stack item
    table[DUP14 as usize] = stack::dup::<14, _, _>; // 0x8D: Duplicate 14th stack item
    table[DUP15 as usize] = stack::dup::<15, _, _>; // 0x8E: Duplicate 15th stack item
    table[DUP16 as usize] = stack::dup::<16, _, _>; // 0x8F: Duplicate 16th stack item

    // === Swap Operations (0x90-0x9F) ===
    // Swap the top stack item with items at various depths
    table[SWAP1 as usize] = stack::swap::<1, _, _>; // 0x90: Swap 1st and 2nd stack items
    table[SWAP2 as usize] = stack::swap::<2, _, _>; // 0x91: Swap 1st and 3rd stack items
    table[SWAP3 as usize] = stack::swap::<3, _, _>; // 0x92: Swap 1st and 4th stack items
    table[SWAP4 as usize] = stack::swap::<4, _, _>; // 0x93: Swap 1st and 5th stack items
    table[SWAP5 as usize] = stack::swap::<5, _, _>; // 0x94: Swap 1st and 6th stack items
    table[SWAP6 as usize] = stack::swap::<6, _, _>; // 0x95: Swap 1st and 7th stack items
    table[SWAP7 as usize] = stack::swap::<7, _, _>; // 0x96: Swap 1st and 8th stack items
    table[SWAP8 as usize] = stack::swap::<8, _, _>; // 0x97: Swap 1st and 9th stack items
    table[SWAP9 as usize] = stack::swap::<9, _, _>; // 0x98: Swap 1st and 10th stack items
    table[SWAP10 as usize] = stack::swap::<10, _, _>; // 0x99: Swap 1st and 11th stack items
    table[SWAP11 as usize] = stack::swap::<11, _, _>; // 0x9A: Swap 1st and 12th stack items
    table[SWAP12 as usize] = stack::swap::<12, _, _>; // 0x9B: Swap 1st and 13th stack items
    table[SWAP13 as usize] = stack::swap::<13, _, _>; // 0x9C: Swap 1st and 14th stack items
    table[SWAP14 as usize] = stack::swap::<14, _, _>; // 0x9D: Swap 1st and 15th stack items
    table[SWAP15 as usize] = stack::swap::<15, _, _>; // 0x9E: Swap 1st and 16th stack items
    table[SWAP16 as usize] = stack::swap::<16, _, _>; // 0x9F: Swap 1st and 17th stack items

    // === Logging Operations (0xA0-0xA4) ===
    // Emit events with varying numbers of indexed topics
    table[LOG0 as usize] = host::log::<0, _>; // 0xA0: Log with 0 topics [HOST]
    table[LOG1 as usize] = host::log::<1, _>; // 0xA1: Log with 1 topic [HOST]
    table[LOG2 as usize] = host::log::<2, _>; // 0xA2: Log with 2 topics [HOST]
    table[LOG3 as usize] = host::log::<3, _>; // 0xA3: Log with 3 topics [HOST]
    table[LOG4 as usize] = host::log::<4, _>; // 0xA4: Log with 4 topics [HOST]

    // === EOF (Ethereum Object Format) Operations (0xD0-0xEF) ===
    // Data operations for EOF contracts
    table[DATALOAD as usize] = data::data_load; // 0xD0: Load word from data section
    table[DATALOADN as usize] = data::data_loadn; // 0xD1: Load word from data section (immediate)
    table[DATASIZE as usize] = data::data_size; // 0xD2: Get size of data section
    table[DATACOPY as usize] = data::data_copy; // 0xD3: Copy data section to memory

    // EOF control flow operations
    table[RJUMP as usize] = control::rjump; // 0xE0: Relative jump
    table[RJUMPI as usize] = control::rjumpi; // 0xE1: Conditional relative jump
    table[RJUMPV as usize] = control::rjumpv; // 0xE2: Relative jump via jump table
    table[CALLF as usize] = control::callf; // 0xE3: Call function
    table[RETF as usize] = control::retf; // 0xE4: Return from function
    table[JUMPF as usize] = control::jumpf; // 0xE5: Jump to function
    table[DUPN as usize] = stack::dupn; // 0xE6: Duplicate nth stack item
    table[SWAPN as usize] = stack::swapn; // 0xE7: Swap top with nth stack item
    table[EXCHANGE as usize] = stack::exchange; // 0xE8: Exchange stack items

    // EOF contract operations
    table[EOFCREATE as usize] = contract::eofcreate; // 0xEC: Create EOF contract
    table[RETURNCONTRACT as usize] = contract::return_contract; // 0xEE: Return contract code

    // === System and Contract Operations (0xF0-0xFF) ===
    // Contract creation and management
    table[CREATE as usize] = contract::create::<_, false, _>; // 0xF0: Create contract [HOST]
    table[CALL as usize] = contract::call; // 0xF1: Message call [HOST]
    table[CALLCODE as usize] = contract::call_code; // 0xF2: Message call with alternative code [HOST]
    table[RETURN as usize] = control::ret; // 0xF3: Halt and return data
    table[DELEGATECALL as usize] = contract::delegate_call; // 0xF4: Message call with sender and value [HOST]
    table[CREATE2 as usize] = contract::create::<_, true, _>; // 0xF5: Create contract with salt [HOST]

    // Advanced system operations
    table[RETURNDATALOAD as usize] = system::returndataload; // 0xF7: Load word from return data
    table[EXTCALL as usize] = contract::extcall; // 0xF8: External call [HOST]
    table[EXTDELEGATECALL as usize] = contract::extdelegatecall; // 0xF9: External delegate call [HOST]
    table[STATICCALL as usize] = contract::static_call; // 0xFA: Static message call [HOST]
    table[EXTSTATICCALL as usize] = contract::extstaticcall; // 0xFB: External static call [HOST]
    table[REVERT as usize] = control::revert; // 0xFD: Halt and revert state changes
    table[INVALID as usize] = control::invalid; // 0xFE: Invalid instruction
    table[SELFDESTRUCT as usize] = host::selfdestruct; // 0xFF: Self-destruct contract [HOST]

    table
}
