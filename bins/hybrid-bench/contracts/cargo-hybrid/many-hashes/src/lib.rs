#![no_std]
#![no_main]

use core::default::Default;

use hybrid_derive::{contract, storage, Event, Error};
use hybrid_contract::hstd::*;

use alloy_core::primitives::{keccak256 as alloy_keccak256, Address, B256, U256};

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
    
    pub fn Benchmark(&mut self, n: U256) -> Result<B256, StorageError> {
        let nn = n.into_limbs()[0];
        let mut out = B256::ZERO;
        
        for i in 0..nn {
            out = alloy_keccak256(i.to_be_bytes())
        }

        Ok(out)
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
