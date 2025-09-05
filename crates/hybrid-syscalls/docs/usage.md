# Usage Guide - Hybrid Syscalls

This guide provides detailed information on how to use the `hybrid-syscalls` crate for interfacing between RISC-V smart contracts and EVM blockchain environments.

## Table of Contents

- [Getting Started](#getting-started)
- [Basic Usage](#basic-usage)
- [Syscall Categories](#syscall-categories)
- [Integration Patterns](#integration-patterns)
- [Error Handling](#error-handling)
- [Performance Considerations](#performance-considerations)
- [Debugging](#debugging)

## Getting Started

### Installation

Add the crate to your `Cargo.toml`:

```toml
[dependencies]
hybrid-syscalls = "0.1.0"
```

### Basic Imports

```rust
use hybrid_syscalls::{Syscall, Error};
use std::str::FromStr;
```

## Basic Usage

### Converting Between Representations

The `Syscall` enum provides seamless conversion between different representations:

```rust
use hybrid_syscalls::Syscall;

// From opcode (u8) to syscall
let syscall = Syscall::try_from(0x33)?; // Caller
assert_eq!(syscall, Syscall::Caller);

// From syscall to opcode
let opcode: u8 = Syscall::SLoad.into();
assert_eq!(opcode, 0x54);

// From string to syscall
let syscall = Syscall::from_str("keccak256")?;
assert_eq!(syscall, Syscall::Keccak256);

// From syscall to string
let name = format!("{}", Syscall::Call);
assert_eq!(name, "call");
```

### Pattern Matching

Use pattern matching for control flow based on syscalls:

```rust
fn handle_syscall(opcode: u8) -> Result<(), Error> {
    let syscall = Syscall::try_from(opcode)?;
    
    match syscall {
        Syscall::Caller => {
            println!("Getting caller address");
            // Handle caller syscall
        },
        Syscall::SLoad => {
            println!("Loading from storage");
            // Handle storage load
        },
        Syscall::Call => {
            println!("Making external call");
            // Handle contract call
        },
        _ => {
            println!("Other syscall: {}", syscall);
        }
    }
    
    Ok(())
}
```

## Syscall Categories

### Environment Information

These syscalls provide information about the current execution context:

```rust
// Get current execution environment info
match syscall {
    Syscall::Origin => {
        // Returns the transaction originator address
        // Equivalent to EVM ORIGIN opcode (0x32)
    },
    Syscall::Caller => {
        // Returns the immediate caller address  
        // Equivalent to EVM CALLER opcode (0x33)
    },
    Syscall::CallValue => {
        // Returns the wei value sent with the current call
        // Equivalent to EVM CALLVALUE opcode (0x34)
    },
    Syscall::GasPrice => {
        // Returns the gas price of the transaction
        // Equivalent to EVM GASPRICE opcode (0x3A)
    },
    _ => {}
}
```

### Block Information

Access current block data:

```rust
match syscall {
    Syscall::Timestamp => {
        // Current block timestamp
        // Equivalent to EVM TIMESTAMP opcode (0x42)
    },
    Syscall::Number => {
        // Current block number
        // Equivalent to EVM NUMBER opcode (0x43)
    },
    Syscall::GasLimit => {
        // Current block gas limit
        // Equivalent to EVM GASLIMIT opcode (0x45)
    },
    Syscall::ChainId => {
        // Current chain ID
        // Equivalent to EVM CHAINID opcode (0x46)
    },
    Syscall::BaseFee => {
        // Current block base fee
        // Equivalent to EVM BASEFEE opcode (0x48)
    },
    _ => {}
}
```

### Storage Operations

Persistent storage access:

```rust
match syscall {
    Syscall::SLoad => {
        // Load 32 bytes from contract storage
        // Parameters: 32-byte storage key
        // Returns: 32-byte storage value
        // Equivalent to EVM SLOAD opcode (0x54)
    },
    Syscall::SStore => {
        // Store 32 bytes to contract storage
        // Parameters: 32-byte key, 32-byte value
        // Equivalent to EVM SSTORE opcode (0x55)
    },
    _ => {}
}
```

### Cryptographic Operations

```rust
match syscall {
    Syscall::Keccak256 => {
        // Compute Keccak-256 hash
        // Parameters: memory offset, data size
        // Returns: 32-byte hash
        // Equivalent to EVM KECCAK256 opcode (0x20)
    },
    _ => {}
}
```

### Contract Interaction

```rust
match syscall {
    Syscall::Call => {
        // Call another contract
        // Parameters: address, value, calldata offset, calldata size
        // Returns: success flag and return data
        // Equivalent to EVM CALL opcode (0xF1)
    },
    Syscall::StaticCall => {
        // Static call (read-only) to another contract
        // Parameters: address, calldata offset, calldata size
        // Returns: success flag and return data
        // Equivalent to EVM STATICCALL opcode (0xFA)
    },
    Syscall::Create => {
        // Create new contract
        // Parameters: value, calldata offset, calldata size
        // Returns: new contract address
        // Equivalent to EVM CREATE opcode (0xF0)
    },
    _ => {}
}
```

### Control Flow

```rust
match syscall {
    Syscall::Return => {
        // Return data and halt execution
        // Parameters: data offset, data size
        // Equivalent to EVM RETURN opcode (0xF3)
    },
    Syscall::Revert => {
        // Revert transaction and halt execution
        // Equivalent to EVM REVERT opcode (0xFD)
    },
    _ => {}
}
```

## Integration Patterns

### VM Emulator Integration

When integrating with a RISC-V emulator for smart contract execution:

```rust
use hybrid_syscalls::Syscall;

struct ContractVM {
    // VM state
}

impl ContractVM {
    fn handle_ecall(&mut self, t0: u8, args: &[u64]) -> Result<u64, Error> {
        let syscall = Syscall::try_from(t0)?;
        
        match syscall {
            Syscall::Caller => {
                // Return caller address from execution context
                Ok(self.get_caller_address())
            },
            Syscall::SLoad => {
                // Load from storage using args[0] as key
                let key = args[0];
                Ok(self.storage_load(key))
            },
            Syscall::SStore => {
                // Store to storage using args[0] as key, args[1] as value
                let key = args[0];
                let value = args[1];
                self.storage_store(key, value);
                Ok(0)
            },
            _ => {
                // Handle other syscalls...
                self.handle_other_syscall(syscall, args)
            }
        }
    }
    
    fn get_caller_address(&self) -> u64 {
        // Implementation specific
        0x1234567890abcdef
    }
    
    fn storage_load(&self, key: u64) -> u64 {
        // Load from contract storage
        0
    }
    
    fn storage_store(&mut self, key: u64, value: u64) {
        // Store to contract storage
    }
    
    fn handle_other_syscall(&mut self, syscall: Syscall, args: &[u64]) -> Result<u64, Error> {
        // Handle remaining syscalls
        Ok(0)
    }
}
```

### Syscall Validation

Validate syscall sequences for security:

```rust
struct SyscallValidator {
    allowed_syscalls: Vec<Syscall>,
    read_only_mode: bool,
}

impl SyscallValidator {
    fn validate_syscall(&self, syscall: Syscall) -> Result<(), Error> {
        // Check if syscall is allowed
        if !self.allowed_syscalls.contains(&syscall) {
            return Err(Error::UnknownOpcode(syscall.into()));
        }
        
        // Check read-only constraints
        if self.read_only_mode {
            match syscall {
                Syscall::SStore | Syscall::Call | Syscall::Create => {
                    return Err(Error::ParseError { 
                        input: "Write operation not allowed in read-only mode".into() 
                    });
                },
                _ => {}
            }
        }
        
        Ok(())
    }
}
```

### Syscall Logging

Log syscall execution for debugging and analysis:

```rust
use std::collections::HashMap;

struct SyscallTracer {
    call_counts: HashMap<Syscall, u64>,
    execution_trace: Vec<(Syscall, u64)>, // (syscall, timestamp)
}

impl SyscallTracer {
    fn trace_syscall(&mut self, opcode: u8) -> Result<(), Error> {
        let syscall = Syscall::try_from(opcode)?;
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        
        // Update call count
        *self.call_counts.entry(syscall).or_insert(0) += 1;
        
        // Add to execution trace
        self.execution_trace.push((syscall, timestamp));
        
        println!("SYSCALL: {} (0x{:02x}) at {}", syscall, opcode, timestamp);
        
        Ok(())
    }
    
    fn get_statistics(&self) -> HashMap<Syscall, u64> {
        self.call_counts.clone()
    }
}
```

## Error Handling

### Comprehensive Error Handling

```rust
use hybrid_syscalls::{Syscall, Error};

fn parse_syscalls(opcodes: &[u8]) -> Result<Vec<Syscall>, Error> {
    let mut syscalls = Vec::new();
    
    for &opcode in opcodes {
        match Syscall::try_from(opcode) {
            Ok(syscall) => syscalls.push(syscall),
            Err(Error::UnknownOpcode(code)) => {
                eprintln!("Warning: Unknown opcode 0x{:02x}, skipping", code);
                continue;
            },
            Err(e) => return Err(e),
        }
    }
    
    Ok(syscalls)
}

fn parse_syscall_names(names: &[&str]) -> Result<Vec<Syscall>, Vec<String>> {
    let mut syscalls = Vec::new();
    let mut errors = Vec::new();
    
    for &name in names {
        match Syscall::from_str(name) {
            Ok(syscall) => syscalls.push(syscall),
            Err(Error::ParseError { input }) => {
                errors.push(format!("Invalid syscall name: {}", input));
            },
            Err(e) => {
                errors.push(format!("Parse error: {}", e));
            }
        }
    }
    
    if errors.is_empty() {
        Ok(syscalls)
    } else {
        Err(errors)
    }
}
```

### Error Recovery Strategies

```rust
fn robust_syscall_handling(opcode: u8) {
    match Syscall::try_from(opcode) {
        Ok(syscall) => {
            println!("Processing syscall: {}", syscall);
            // Handle valid syscall
        },
        Err(Error::UnknownOpcode(code)) => {
            // Attempt recovery or fallback
            eprintln!("Unknown opcode 0x{:02x}, attempting fallback", code);
            handle_unknown_syscall(code);
        },
        Err(e) => {
            eprintln!("Syscall parsing error: {}", e);
            // Log error and continue
        }
    }
}

fn handle_unknown_syscall(opcode: u8) {
    // Implement fallback logic for unknown syscalls
    match opcode {
        0x00..=0x1f => {
            // Might be a future syscall, ignore for now
            eprintln!("Ignoring potential future syscall: 0x{:02x}", opcode);
        },
        _ => {
            // Truly unknown, might be an error
            eprintln!("Unrecognized opcode: 0x{:02x}", opcode);
        }
    }
}
```

## Performance Considerations

### Efficient Syscall Processing

```rust
use hybrid_syscalls::Syscall;
use std::collections::HashSet;

// Pre-compute sets for fast lookups
lazy_static::lazy_static! {
    static ref STORAGE_SYSCALLS: HashSet<Syscall> = {
        let mut set = HashSet::new();
        set.insert(Syscall::SLoad);
        set.insert(Syscall::SStore);
        set
    };
    
    static ref READONLY_SYSCALLS: HashSet<Syscall> = {
        let mut set = HashSet::new();
        set.insert(Syscall::Origin);
        set.insert(Syscall::Caller);
        set.insert(Syscall::CallValue);
        set.insert(Syscall::SLoad);
        set.insert(Syscall::StaticCall);
        set
    };
}

fn is_storage_operation(syscall: Syscall) -> bool {
    STORAGE_SYSCALLS.contains(&syscall)
}

fn is_readonly_operation(syscall: Syscall) -> bool {
    READONLY_SYSCALLS.contains(&syscall)
}

// Batch processing for better performance
fn process_syscall_batch(opcodes: &[u8]) -> Result<Vec<Syscall>, Error> {
    opcodes.iter()
        .map(|&opcode| Syscall::try_from(opcode))
        .collect()
}
```

### Memory-Efficient Representations

```rust
// Use opcode directly when possible to avoid allocations
fn syscall_id_only(opcode: u8) -> u8 {
    // Just return the opcode without creating the enum
    opcode
}

// Batch convert only when needed
fn lazy_syscall_conversion(opcodes: &[u8]) -> impl Iterator<Item = Result<Syscall, Error>> + '_ {
    opcodes.iter().map(|&opcode| Syscall::try_from(opcode))
}
```

## Debugging

### Syscall Debugging Tools

```rust
use hybrid_syscalls::Syscall;

struct SyscallDebugger {
    breakpoints: HashSet<Syscall>,
    step_mode: bool,
}

impl SyscallDebugger {
    fn new() -> Self {
        Self {
            breakpoints: HashSet::new(),
            step_mode: false,
        }
    }
    
    fn add_breakpoint(&mut self, syscall: Syscall) {
        self.breakpoints.insert(syscall);
    }
    
    fn should_break(&self, syscall: Syscall) -> bool {
        self.step_mode || self.breakpoints.contains(&syscall)
    }
    
    fn debug_syscall(&self, opcode: u8, args: &[u64]) -> Result<(), Error> {
        let syscall = Syscall::try_from(opcode)?;
        
        if self.should_break(syscall) {
            println!("BREAKPOINT: {} (0x{:02x})", syscall, opcode);
            println!("Arguments: {:?}", args);
            
            // Wait for user input in step mode
            if self.step_mode {
                println!("Press Enter to continue...");
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).unwrap();
            }
        }
        
        Ok(())
    }
}

// Usage
fn debug_contract_execution(opcodes: &[u8]) {
    let mut debugger = SyscallDebugger::new();
    debugger.add_breakpoint(Syscall::SStore);
    debugger.add_breakpoint(Syscall::Call);
    
    for &opcode in opcodes {
        let args = [0u64; 8]; // Mock arguments
        if let Err(e) = debugger.debug_syscall(opcode, &args) {
            eprintln!("Debug error: {}", e);
        }
    }
}
```

### Syscall Analysis

```rust
use std::collections::HashMap;

fn analyze_syscall_usage(contract_bytecode: &[u8]) -> Result<(), Error> {
    let mut syscall_stats = HashMap::new();
    let mut complexity_score = 0;
    
    for &opcode in contract_bytecode {
        if let Ok(syscall) = Syscall::try_from(opcode) {
            *syscall_stats.entry(syscall).or_insert(0) += 1;
            
            // Assign complexity scores
            complexity_score += match syscall {
                Syscall::Caller | Syscall::Origin => 1,
                Syscall::SLoad | Syscall::SStore => 2,
                Syscall::Call | Syscall::StaticCall => 3,
                Syscall::Create => 5,
                _ => 1,
            };
        }
    }
    
    println!("Syscall Usage Analysis:");
    for (syscall, count) in &syscall_stats {
        println!("  {}: {} times", syscall, count);
    }
    println!("Complexity Score: {}", complexity_score);
    
    Ok(())
}
```

This guide covers the essential patterns and best practices for using the `hybrid-syscalls` crate effectively in blockchain and smart contract development contexts.