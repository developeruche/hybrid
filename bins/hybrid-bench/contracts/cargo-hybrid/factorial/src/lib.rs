#![no_std]
#![no_main]

use core::default::Default;

use hybrid_contract::hstd::*;
use hybrid_derive::{contract, storage, Error, Event};

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
    FailedToSetStorage,
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

    pub fn Benchmark(&mut self, n: U256) -> Result<U256, StorageError> {
        let nn = n.into_limbs()[0];
        if nn == 0 || nn == 1 {
            return Ok(U256::from(1));
        }

        let mut result: u64 = 1;
        for i in 2..=nn {
            // Check for overflow: result * i <= u64::MAX
            if result > (u64::MAX / i) {
                return Ok(U256::from(u64::MAX));
            } else {
                result *= i;
            }
        }

        Ok(U256::from(result))
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
