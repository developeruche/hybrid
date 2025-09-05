# Usage Examples - Hybrid Derive

This document provides comprehensive examples demonstrating how to use the `hybrid-derive` crate to build smart contracts.

## Table of Contents

- [Basic ERC20 Token](#basic-erc20-token)
- [NFT Contract](#nft-contract)
- [Multi-signature Wallet](#multi-signature-wallet)
- [Governance Contract](#governance-contract)
- [Advanced Features](#advanced-features)

## Basic ERC20 Token

A complete implementation of an ERC20 fungible token contract.

```rust
use hybrid_derive::{contract, storage, Error, Event, payable};
use alloy_primitives::{Address, U256, String};
use hybrid_contract::*;

#[storage]
pub struct TokenStorage {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub total_supply: U256,
    pub balances: StorageMap<Address, U256>,
    pub allowances: StorageMap<Address, StorageMap<Address, U256>>,
}

#[derive(Error)]
pub enum TokenError {
    InsufficientBalance,
    InsufficientAllowance,
    InvalidAddress,
    TransferToSelf,
}

#[derive(Event)]
pub struct Transfer {
    #[indexed]
    pub from: Address,
    #[indexed]
    pub to: Address,
    pub value: U256,
}

#[derive(Event)]
pub struct Approval {
    #[indexed]
    pub owner: Address,
    #[indexed]
    pub spender: Address,
    pub value: U256,
}

#[contract]
impl TokenStorage {
    pub fn new(name: String, symbol: String, initial_supply: U256) -> Self {
        let mut storage = Self::default();
        let deployer = caller();
        
        storage.name = name;
        storage.symbol = symbol;
        storage.decimals = 18;
        storage.total_supply = initial_supply;
        storage.balances.set(deployer, initial_supply);
        
        emit!(Transfer, Address::ZERO, deployer, initial_supply);
        storage
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn symbol(&self) -> String {
        self.symbol.clone()
    }

    pub fn decimals(&self) -> u8 {
        self.decimals
    }

    pub fn total_supply(&self) -> U256 {
        self.total_supply
    }

    pub fn balance_of(&self, account: Address) -> U256 {
        self.balances.get(account).unwrap_or(U256::ZERO)
    }

    pub fn allowance(&self, owner: Address, spender: Address) -> U256 {
        self.allowances
            .get(owner)
            .and_then(|owner_allowances| owner_allowances.get(spender))
            .unwrap_or(U256::ZERO)
    }

    pub fn transfer(&mut self, to: Address, amount: U256) -> Result<bool, TokenError> {
        let from = caller();
        self._transfer(from, to, amount)?;
        Ok(true)
    }

    pub fn approve(&mut self, spender: Address, amount: U256) -> Result<bool, TokenError> {
        let owner = caller();
        self._approve(owner, spender, amount)?;
        Ok(true)
    }

    pub fn transfer_from(&mut self, from: Address, to: Address, amount: U256) -> Result<bool, TokenError> {
        let spender = caller();
        let current_allowance = self.allowance(from, spender);
        
        if current_allowance < amount {
            return Err(TokenError::InsufficientAllowance);
        }

        self._transfer(from, to, amount)?;
        self._approve(from, spender, current_allowance - amount)?;
        Ok(true)
    }

    // Internal helper functions
    fn _transfer(&mut self, from: Address, to: Address, amount: U256) -> Result<(), TokenError> {
        if from == to {
            return Err(TokenError::TransferToSelf);
        }

        let from_balance = self.balance_of(from);
        if from_balance < amount {
            return Err(TokenError::InsufficientBalance);
        }

        self.balances.set(from, from_balance - amount);
        let to_balance = self.balance_of(to);
        self.balances.set(to, to_balance + amount);

        emit!(Transfer, from, to, amount);
        Ok(())
    }

    fn _approve(&mut self, owner: Address, spender: Address, amount: U256) -> Result<(), TokenError> {
        let mut owner_allowances = self.allowances.get(owner).unwrap_or_default();
        owner_allowances.set(spender, amount);
        self.allowances.set(owner, owner_allowances);

        emit!(Approval, owner, spender, amount);
        Ok(())
    }
}
```

## NFT Contract

An ERC721-compatible non-fungible token implementation.

```rust
use hybrid_derive::{contract, storage, Error, Event};
use alloy_primitives::{Address, U256, String};
use hybrid_contract::*;

#[storage]
pub struct NFTStorage {
    pub name: String,
    pub symbol: String,
    pub owners: StorageMap<U256, Address>,
    pub balances: StorageMap<Address, U256>,
    pub token_approvals: StorageMap<U256, Address>,
    pub operator_approvals: StorageMap<Address, StorageMap<Address, bool>>,
    pub token_uris: StorageMap<U256, String>,
    pub next_token_id: U256,
}

#[derive(Error)]
pub enum NFTError {
    TokenNotExists,
    NotOwnerOrApproved,
    ApprovalToCurrentOwner,
    TransferToNonReceiver,
    MintToZeroAddress,
}

#[derive(Event)]
pub struct Transfer {
    #[indexed]
    pub from: Address,
    #[indexed]
    pub to: Address,
    #[indexed]
    pub token_id: U256,
}

#[derive(Event)]
pub struct Approval {
    #[indexed]
    pub owner: Address,
    #[indexed]
    pub approved: Address,
    #[indexed]
    pub token_id: U256,
}

#[derive(Event)]
pub struct ApprovalForAll {
    #[indexed]
    pub owner: Address,
    #[indexed]
    pub operator: Address,
    pub approved: bool,
}

#[contract]
impl NFTStorage {
    pub fn new(name: String, symbol: String) -> Self {
        let mut storage = Self::default();
        storage.name = name;
        storage.symbol = symbol;
        storage.next_token_id = U256::from(1);
        storage
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn symbol(&self) -> String {
        self.symbol.clone()
    }

    pub fn balance_of(&self, owner: Address) -> U256 {
        self.balances.get(owner).unwrap_or(U256::ZERO)
    }

    pub fn owner_of(&self, token_id: U256) -> Result<Address, NFTError> {
        self.owners.get(token_id).ok_or(NFTError::TokenNotExists)
    }

    pub fn token_uri(&self, token_id: U256) -> Result<String, NFTError> {
        if !self._exists(token_id) {
            return Err(NFTError::TokenNotExists);
        }
        Ok(self.token_uris.get(token_id).unwrap_or_default())
    }

    pub fn approve(&mut self, to: Address, token_id: U256) -> Result<(), NFTError> {
        let owner = self.owner_of(token_id)?;
        let caller = caller();

        if to == owner {
            return Err(NFTError::ApprovalToCurrentOwner);
        }

        if caller != owner && !self.is_approved_for_all(owner, caller) {
            return Err(NFTError::NotOwnerOrApproved);
        }

        self.token_approvals.set(token_id, to);
        emit!(Approval, owner, to, token_id);
        Ok(())
    }

    pub fn get_approved(&self, token_id: U256) -> Result<Address, NFTError> {
        if !self._exists(token_id) {
            return Err(NFTError::TokenNotExists);
        }
        Ok(self.token_approvals.get(token_id).unwrap_or(Address::ZERO))
    }

    pub fn set_approval_for_all(&mut self, operator: Address, approved: bool) {
        let owner = caller();
        let mut owner_approvals = self.operator_approvals.get(owner).unwrap_or_default();
        owner_approvals.set(operator, approved);
        self.operator_approvals.set(owner, owner_approvals);

        emit!(ApprovalForAll, owner, operator, approved);
    }

    pub fn is_approved_for_all(&self, owner: Address, operator: Address) -> bool {
        self.operator_approvals
            .get(owner)
            .and_then(|approvals| approvals.get(operator))
            .unwrap_or(false)
    }

    pub fn transfer_from(&mut self, from: Address, to: Address, token_id: U256) -> Result<(), NFTError> {
        if !self._is_approved_or_owner(caller(), token_id)? {
            return Err(NFTError::NotOwnerOrApproved);
        }
        self._transfer(from, to, token_id)
    }

    pub fn mint(&mut self, to: Address, uri: String) -> Result<U256, NFTError> {
        let token_id = self.next_token_id;
        self._mint(to, token_id, uri)?;
        self.next_token_id = token_id + U256::from(1);
        Ok(token_id)
    }

    // Internal helper functions
    fn _exists(&self, token_id: U256) -> bool {
        self.owners.get(token_id).is_some()
    }

    fn _is_approved_or_owner(&self, spender: Address, token_id: U256) -> Result<bool, NFTError> {
        let owner = self.owner_of(token_id)?;
        Ok(spender == owner || 
           self.get_approved(token_id)? == spender ||
           self.is_approved_for_all(owner, spender))
    }

    fn _mint(&mut self, to: Address, token_id: U256, uri: String) -> Result<(), NFTError> {
        if to == Address::ZERO {
            return Err(NFTError::MintToZeroAddress);
        }

        self.owners.set(token_id, to);
        self.token_uris.set(token_id, uri);
        
        let balance = self.balance_of(to);
        self.balances.set(to, balance + U256::from(1));

        emit!(Transfer, Address::ZERO, to, token_id);
        Ok(())
    }

    fn _transfer(&mut self, from: Address, to: Address, token_id: U256) -> Result<(), NFTError> {
        let owner = self.owner_of(token_id)?;
        if owner != from {
            return Err(NFTError::NotOwnerOrApproved);
        }

        // Clear approvals
        self.token_approvals.set(token_id, Address::ZERO);

        // Update balances
        let from_balance = self.balance_of(from);
        self.balances.set(from, from_balance - U256::from(1));
        
        let to_balance = self.balance_of(to);
        self.balances.set(to, to_balance + U256::from(1));

        // Update owner
        self.owners.set(token_id, to);

        emit!(Transfer, from, to, token_id);
        Ok(())
    }
}
```

## Multi-signature Wallet

A wallet that requires multiple signatures to execute transactions.

```rust
use hybrid_derive::{contract, storage, Error, Event};
use alloy_primitives::{Address, U256, Bytes};
use hybrid_contract::*;

#[storage]
pub struct MultiSigStorage {
    pub owners: Vec<Address>,
    pub required: U256,
    pub transactions: StorageMap<U256, Transaction>,
    pub confirmations: StorageMap<U256, StorageMap<Address, bool>>,
    pub transaction_count: U256,
}

#[derive(Clone)]
pub struct Transaction {
    pub to: Address,
    pub value: U256,
    pub data: Bytes,
    pub executed: bool,
}

#[derive(Error)]
pub enum MultiSigError {
    NotOwner,
    TransactionNotExists,
    AlreadyConfirmed,
    NotConfirmed,
    AlreadyExecuted,
    InsufficientConfirmations,
    ExecutionFailed,
}

#[derive(Event)]
pub struct Submission {
    #[indexed]
    pub transaction_id: U256,
}

#[derive(Event)]
pub struct Confirmation {
    #[indexed]
    pub owner: Address,
    #[indexed]
    pub transaction_id: U256,
}

#[derive(Event)]
pub struct Execution {
    #[indexed]
    pub transaction_id: U256,
}

#[contract]
impl MultiSigStorage {
    pub fn new(owners: Vec<Address>, required: U256) -> Self {
        let mut storage = Self::default();
        storage.owners = owners;
        storage.required = required;
        storage.transaction_count = U256::ZERO;
        storage
    }

    #[payable]
    pub fn receive(&mut self) {
        // Accept ether deposits
    }

    pub fn submit_transaction(&mut self, to: Address, value: U256, data: Bytes) -> Result<U256, MultiSigError> {
        if !self.is_owner(caller()) {
            return Err(MultiSigError::NotOwner);
        }

        let transaction_id = self.transaction_count;
        let transaction = Transaction {
            to,
            value,
            data,
            executed: false,
        };

        self.transactions.set(transaction_id, transaction);
        self.transaction_count = transaction_id + U256::from(1);

        emit!(Submission, transaction_id);
        Ok(transaction_id)
    }

    pub fn confirm_transaction(&mut self, transaction_id: U256) -> Result<(), MultiSigError> {
        let caller = caller();
        if !self.is_owner(caller) {
            return Err(MultiSigError::NotOwner);
        }

        if !self.transaction_exists(transaction_id) {
            return Err(MultiSigError::TransactionNotExists);
        }

        if self.is_confirmed(transaction_id, caller) {
            return Err(MultiSigError::AlreadyConfirmed);
        }

        let mut tx_confirmations = self.confirmations.get(transaction_id).unwrap_or_default();
        tx_confirmations.set(caller, true);
        self.confirmations.set(transaction_id, tx_confirmations);

        emit!(Confirmation, caller, transaction_id);
        Ok(())
    }

    pub fn execute_transaction(&mut self, transaction_id: U256) -> Result<(), MultiSigError> {
        if !self.transaction_exists(transaction_id) {
            return Err(MultiSigError::TransactionNotExists);
        }

        let mut transaction = self.transactions.get(transaction_id).unwrap();
        if transaction.executed {
            return Err(MultiSigError::AlreadyExecuted);
        }

        if !self.is_confirmed_by_required(transaction_id) {
            return Err(MultiSigError::InsufficientConfirmations);
        }

        transaction.executed = true;
        self.transactions.set(transaction_id, transaction.clone());

        // Execute the transaction
        let success = call_contract(
            transaction.to,
            transaction.value.as_u64(),
            &transaction.data,
            Some(gas_left())
        );

        if success.is_empty() {
            return Err(MultiSigError::ExecutionFailed);
        }

        emit!(Execution, transaction_id);
        Ok(())
    }

    pub fn is_confirmed_by_required(&self, transaction_id: U256) -> bool {
        let confirmations = self.get_confirmation_count(transaction_id);
        confirmations >= self.required
    }

    pub fn get_confirmation_count(&self, transaction_id: U256) -> U256 {
        let tx_confirmations = self.confirmations.get(transaction_id).unwrap_or_default();
        let mut count = U256::ZERO;
        
        for owner in &self.owners {
            if tx_confirmations.get(*owner).unwrap_or(false) {
                count = count + U256::from(1);
            }
        }
        
        count
    }

    // Helper functions
    fn is_owner(&self, addr: Address) -> bool {
        self.owners.contains(&addr)
    }

    fn transaction_exists(&self, transaction_id: U256) -> bool {
        self.transactions.get(transaction_id).is_some()
    }

    fn is_confirmed(&self, transaction_id: U256, owner: Address) -> bool {
        self.confirmations
            .get(transaction_id)
            .and_then(|confirmations| confirmations.get(owner))
            .unwrap_or(false)
    }
}
```

## Governance Contract

A decentralized governance system with proposal creation and voting.

```rust
use hybrid_derive::{contract, storage, Error, Event};
use alloy_primitives::{Address, U256, String};
use hybrid_contract::*;

#[storage]
pub struct GovernanceStorage {
    pub token: Address,
    pub proposals: StorageMap<U256, Proposal>,
    pub votes: StorageMap<U256, StorageMap<Address, Vote>>,
    pub proposal_count: U256,
    pub voting_period: U256,
    pub quorum: U256,
}

#[derive(Clone)]
pub struct Proposal {
    pub proposer: Address,
    pub description: String,
    pub start_block: U256,
    pub end_block: U256,
    pub for_votes: U256,
    pub against_votes: U256,
    pub executed: bool,
}

#[derive(Clone)]
pub struct Vote {
    pub support: bool,
    pub votes: U256,
}

#[derive(Error)]
pub enum GovernanceError {
    InsufficientVotingPower,
    ProposalNotActive,
    AlreadyVoted,
    ProposalNotSucceeded,
    AlreadyExecuted,
}

#[derive(Event)]
pub struct ProposalCreated {
    #[indexed]
    pub proposal_id: U256,
    #[indexed]
    pub proposer: Address,
    pub description: String,
}

#[derive(Event)]
pub struct VoteCast {
    #[indexed]
    pub voter: Address,
    #[indexed]
    pub proposal_id: U256,
    pub support: bool,
    pub votes: U256,
}

#[derive(Event)]
pub struct ProposalExecuted {
    #[indexed]
    pub proposal_id: U256,
}

#[contract]
impl GovernanceStorage {
    pub fn new(token: Address, voting_period: U256, quorum: U256) -> Self {
        let mut storage = Self::default();
        storage.token = token;
        storage.voting_period = voting_period;
        storage.quorum = quorum;
        storage.proposal_count = U256::ZERO;
        storage
    }

    pub fn propose(&mut self, description: String) -> Result<U256, GovernanceError> {
        let proposer = caller();
        let voting_power = self.get_voting_power(proposer);
        
        if voting_power == U256::ZERO {
            return Err(GovernanceError::InsufficientVotingPower);
        }

        let proposal_id = self.proposal_count;
        let current_block = block_number();
        
        let proposal = Proposal {
            proposer,
            description: description.clone(),
            start_block: current_block,
            end_block: current_block + self.voting_period,
            for_votes: U256::ZERO,
            against_votes: U256::ZERO,
            executed: false,
        };

        self.proposals.set(proposal_id, proposal);
        self.proposal_count = proposal_id + U256::from(1);

        emit!(ProposalCreated, proposal_id, proposer, description);
        Ok(proposal_id)
    }

    pub fn vote(&mut self, proposal_id: U256, support: bool) -> Result<(), GovernanceError> {
        let voter = caller();
        let proposal = self.proposals.get(proposal_id).unwrap();
        let current_block = block_number();

        if current_block < proposal.start_block || current_block > proposal.end_block {
            return Err(GovernanceError::ProposalNotActive);
        }

        let proposal_votes = self.votes.get(proposal_id).unwrap_or_default();
        if proposal_votes.get(voter).is_some() {
            return Err(GovernanceError::AlreadyVoted);
        }

        let voting_power = self.get_voting_power(voter);
        let vote = Vote {
            support,
            votes: voting_power,
        };

        let mut updated_votes = proposal_votes;
        updated_votes.set(voter, vote);
        self.votes.set(proposal_id, updated_votes);

        // Update proposal vote counts
        let mut updated_proposal = proposal;
        if support {
            updated_proposal.for_votes = updated_proposal.for_votes + voting_power;
        } else {
            updated_proposal.against_votes = updated_proposal.against_votes + voting_power;
        }
        self.proposals.set(proposal_id, updated_proposal);

        emit!(VoteCast, voter, proposal_id, support, voting_power);
        Ok(())
    }

    pub fn execute(&mut self, proposal_id: U256) -> Result<(), GovernanceError> {
        let mut proposal = self.proposals.get(proposal_id).unwrap();
        
        if !self.proposal_succeeded(proposal_id) {
            return Err(GovernanceError::ProposalNotSucceeded);
        }

        if proposal.executed {
            return Err(GovernanceError::AlreadyExecuted);
        }

        proposal.executed = true;
        self.proposals.set(proposal_id, proposal);

        emit!(ProposalExecuted, proposal_id);
        Ok(())
    }

    pub fn proposal_succeeded(&self, proposal_id: U256) -> bool {
        let proposal = self.proposals.get(proposal_id).unwrap();
        let current_block = block_number();
        
        current_block > proposal.end_block &&
        proposal.for_votes > proposal.against_votes &&
        proposal.for_votes >= self.quorum
    }

    fn get_voting_power(&self, account: Address) -> U256 {
        // Call the token contract to get voting power
        let token_interface = IToken::new(self.token).build();
        token_interface.balance_of(account).unwrap_or(U256::ZERO)
    }
}
```

## Advanced Features

### Interface Usage

Using generated interfaces to interact with other contracts:

```rust
use hybrid_derive::interface;
use alloy_primitives::{Address, U256};

#[interface("camelCase")]
trait IERC20 {
    fn total_supply(&self) -> U256;
    fn balance_of(&self, account: Address) -> U256;
    fn transfer(&mut self, to: Address, amount: U256) -> bool;
}

#[contract]
impl MyContract {
    pub fn transfer_tokens(&mut self, token: Address, to: Address, amount: U256) -> Result<(), Error> {
        let token_contract = IERC20::new(token).build();
        let success = token_contract.transfer(to, amount)?;
        
        if !success {
            return Err(Error::TransferFailed);
        }
        
        Ok(())
    }
}
```

### Custom Error Types with Data

```rust
#[derive(Error)]
pub enum DetailedError {
    InsufficientBalance {
        required: U256,
        available: U256,
    },
    InvalidRecipient(Address),
    TransferFailed,
}
```

### Complex Event Logging

```rust
#[derive(Event)]
pub struct ComplexEvent {
    #[indexed]
    pub user: Address,
    #[indexed]
    pub action_type: U256,
    #[indexed]
    pub asset: Address,
    pub amount: U256,
    pub data: Bytes,
    pub timestamp: U256,
}

// Usage
emit!(ComplexEvent, user_addr, action_id, asset_addr, transfer_amount, call_data, block_timestamp());
```

### Storage Patterns

```rust
#[storage]
pub struct ComplexStorage {
    pub owner: Address,
    pub paused: bool,
    pub user_data: StorageMap<Address, UserInfo>,
    pub nested_mapping: StorageMap<Address, StorageMap<U256, bool>>,
    pub array_data: Vec<U256>,
}

#[derive(Clone)]
pub struct UserInfo {
    pub balance: U256,
    pub last_action: U256,
    pub is_active: bool,
}
```

These examples demonstrate the power and flexibility of the `hybrid-derive` crate for building complex smart contracts with minimal boilerplate while maintaining type safety and EVM compatibility.