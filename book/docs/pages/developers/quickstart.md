---
description: Quickstart 
---
                                      
# Quickstart

In this guide, you will learn how to create a smart contract, compile and deploy to the Hybrid node, also in this Quickstart, a Solidity smart contract would be deployed to the Hybrid node (asserting the backward compatibility claim).

### Installation to deployment

See the [Installation Guide](/developers/installation.md) for more details.

### Create and ERC20 smart contract using RUST

This is relatively easy to as an ERC20 contract is one of the templates.
Using this command, you can create a new project with the ERC20 template:

```bash
cargo hybrid new my-token --template erc20
```

In `erc20/src/lib.rs` you would find the main smart contract logic.

```rust 
#![no_std]
#![no_main]

use alloy_core::primitives::{Address, U256};
use core::default::Default;
use hybrid_contract::hstd::*;
use hybrid_derive::{contract, payable, storage, Error, Event};
extern crate alloc;

#[derive(Event)]
pub struct Transfer {
    #[indexed]
    pub from: Address,
    #[indexed]
    pub to: Address,
    pub amount: U256,
}

#[derive(Event)]
pub struct Approval {
    #[indexed]
    pub owner: Address,
    #[indexed]
    pub spender: Address,
    pub amount: U256,
}

#[derive(Event)]
pub struct OwnershipTransferred {
    #[indexed]
    pub from: Address,
    #[indexed]
    pub to: Address,
}

#[derive(Error)]
pub enum ERC20Error {
    OnlyOwner,
    InsufficientBalance(U256),
    InsufficientAllowance(U256),
    SelfApproval,
    SelfTransfer,
    ZeroAmount,
    ZeroAddress,
}

#[storage]
pub struct ERC20 {
    total_supply: Slot<U256>,
    balance_of: Mapping<Address, Slot<U256>>,
    allowance_of: Mapping<Address, Mapping<Address, Slot<U256>>>,
    owner: Slot<Address>,
}

#[contract]
impl ERC20 {
    pub fn new(owner: Address) -> Self {
        let mut erc20 = ERC20::default();
        erc20.owner.write(owner);
        erc20
    }

    #[payable]
    pub fn mint(&mut self, to: Address, amount: U256) -> Result<bool, ERC20Error> {
        if msg_sender() != self.owner.read() {
            return Err(ERC20Error::OnlyOwner);
        };
        if amount == U256::ZERO {
            return Err(ERC20Error::ZeroAmount);
        };
        if to == Address::ZERO {
            return Err(ERC20Error::ZeroAddress);
        };
        let to_balance = self.balance_of[to].read();
        self.balance_of[to].write(to_balance + amount);
        self.total_supply += amount;
        log::emit(Transfer::new(Address::ZERO, to, amount));
        Ok(true)
    }

    pub fn approve(&mut self, spender: Address, amount: U256) -> Result<bool, ERC20Error> {
        let owner = msg_sender();
        if spender == Address::ZERO {
            return Err(ERC20Error::ZeroAddress);
        };
        if spender == owner {
            return Err(ERC20Error::SelfApproval);
        };
        self.allowance_of[owner][spender].write(amount);
        log::emit(Approval::new(owner, spender, amount));
        Ok(true)
    }

    pub fn transfer(&mut self, to: Address, amount: U256) -> Result<bool, ERC20Error> {
        let from = msg_sender();
        if to == Address::ZERO {
            return Err(ERC20Error::ZeroAddress);
        };
        if amount == U256::ZERO {
            return Err(ERC20Error::ZeroAmount);
        };
        if from == to {
            return Err(ERC20Error::SelfTransfer);
        };
        let from_balance = self.balance_of[from].read();
        let to_balance = self.balance_of[to].read();
        if from_balance < amount {
            return Err(ERC20Error::InsufficientBalance(from_balance));
        }
        self.balance_of[from].write(from_balance - amount);
        self.balance_of[to].write(to_balance + amount);
        log::emit(Transfer::new(from, to, amount));
        Ok(true)
    }

    pub fn transfer_from(
        &mut self,
        from: Address,
        to: Address,
        amount: U256,
    ) -> Result<bool, ERC20Error> {
        let msg_sender = msg_sender();
        if to == Address::ZERO {
            return Err(ERC20Error::ZeroAddress);
        };
        if amount == U256::ZERO {
            return Err(ERC20Error::ZeroAmount);
        };
        if from == to {
            return Err(ERC20Error::SelfTransfer);
        };
        let allowance = self.allowance_of[from][msg_sender].read();
        if allowance < amount {
            return Err(ERC20Error::InsufficientAllowance(allowance));
        };
        let from_balance = self.balance_of[from].read();
        if from_balance < amount {
            return Err(ERC20Error::InsufficientBalance(from_balance));
        };
        self.allowance_of[from][msg_sender].write(allowance - amount);
        self.balance_of[from].write(from_balance - amount);
        let to_balance = self.balance_of[to].read();
        self.balance_of[to].write(to_balance + amount);
        log::emit(Transfer::new(from, to, amount));
        Ok(true)
    }

    pub fn transfer_ownership(&mut self, new_owner: Address) -> Result<bool, ERC20Error> {
        let from = msg_sender();
        if from != self.owner.read() {
            return Err(ERC20Error::OnlyOwner);
        };
        if from == new_owner {
            return Err(ERC20Error::SelfTransfer);
        };
        self.owner.write(new_owner);
        log::emit(OwnershipTransferred::new(from, new_owner));
        Ok(true)
    }

    pub fn owner(&self) -> Address {
        self.owner.read()
    }

    pub fn total_supply(&self) -> U256 {
        self.total_supply.read()
    }

    pub fn balance_of(&self, owner: Address) -> U256 {
        self.balance_of[owner].read()
    }

    pub fn allowance(&self, owner: Address, spender: Address) -> U256 {
        self.allowance_of[owner][spender].read()
    }
}
```

For development purposes, you can use the `--dev` flag to deploy to a local development network:

```bash
cargo hybrid node
```

This would be running at `http://127.0.0.1:8545`, now in the rest of the commands, you can use this as your `RPC_URL` variable.


Next up, you run a build command to compile the smart contract.

### Building Contracts

```bash
cd my-token
cargo hybrid build
```

This compiles your Rust smart contract to RISC-V bytecode and outputs the binary to the `out` directory.

**Options:**
- `--out DIR` - Specify a custom output directory (default: "out")

### Checking Contracts

To check if your contract compiles without generating output:

```bash
cargo hybrid check
```

### Deploying Contracts

```bash
cargo hybrid deploy --rpc [RPC_URL]
```