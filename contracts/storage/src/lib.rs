#![no_std]
#![no_main]

use core::default::Default;

use contract_derive::{contract, storage, Event, Error};
use eth_riscv_runtime::types::*;

use alloy_core::primitives::{Address, U256};

extern crate alloc;

// -- EVENTS -------------------------------------------------------------------
#[derive(Event)]
pub struct StarageSet {
    pub storage_item: U256,
}



// -- ERRORS -------------------------------------------------------------------
#[derive(Error)]
pub enum StorageError {
    FailedToSetStorage
}

// -- CONTRACT -----------------------------------------------------------------
#[storage]
pub struct Storage {
    storage_item: Slot<U256>,
}

#[contract]
impl Storage {
    // -- CONSTRUCTOR ----------------------------------------------------------
    pub fn new(init_item: U256) -> Self {
        // Init the contract
        let mut storage = Storage::default();

        // Update state
        storage.storage_item.write(init_item);

        // Return the initialized contract
        storage
    }

    // -- STATE MODIFYING FUNCTIONS --------------------------------------------
    pub fn set_storage(&mut self, item: U256) -> Result<bool, StorageError> {
        let _from = msg_sender();

        // Update state
        self.storage_item.write(item);

        // Emit event + return 
        log::emit(StarageSet::new(item));
        Ok(true)
    }


    // -- READ-ONLY FUNCTIONS --------------------------------------------------
    pub fn read_item(&self) -> U256 {
        self.storage_item.read()
    }
}
