//! # Storage Slot Implementation
//!
//! This module implements the `Slot<V>` type, which represents a single storage slot
//! in the Hybrid VM's persistent storage system. Each slot can hold any value that
//! implements the Solidity ABI encoding traits.
//!
//! ## Overview
//!
//! A `Slot<V>` is a wrapper around a U256 storage key that provides type-safe access
//! to a single storage location. It automatically handles ABI encoding and decoding
//! when reading from and writing to storage, ensuring compatibility with Ethereum's
//! storage format.
//!
//! ## Features
//!
//! - Type-safe storage operations with automatic ABI encoding/decoding
//! - Arithmetic operations (Add, Sub, etc.) with automatic storage updates
//! - Comparison operations that read from storage
//! - Integration with the storage layout system
//!
//! ## Usage
//!
//! ```rust,no_run
//! use hybrid_contract::hstd::Slot;
//! use alloy_core::primitives::U256;
//!
//! // Create a slot (typically done by storage macros)
//! let counter: Slot<U256> = Slot::default();
//!
//! // Write a value
//! counter.write(U256::from(42));
//!
//! // Read the value
//! let value = counter.read();
//! assert_eq!(value, U256::from(42));
//!
//! // Use arithmetic operations
//! counter += U256::from(1);  // Increments counter in storage
//! let result = counter + U256::from(5);  // Reads and adds without storing
//! ```

use super::*;

use core::ops::{Add, AddAssign, Sub, SubAssign};

/// A type-safe wrapper for a single storage slot in the Hybrid VM.
///
/// `Slot<V>` represents a single 256-bit storage location that can hold any value
/// of type `V` that implements the necessary Solidity ABI encoding traits. The slot
/// automatically handles encoding and decoding when values are read from or written
/// to persistent storage.
///
/// # Type Parameters
/// * `V` - The type of value this slot can store. Must implement `SolValue` and related traits.
///
/// # Storage Layout
/// Each slot occupies exactly one storage location (32 bytes) in the contract's storage.
/// Values are ABI-encoded before storage and decoded after retrieval, ensuring
/// compatibility with Ethereum's storage format.
///
/// # Examples
/// ```rust,no_run
/// use hybrid_contract::hstd::Slot;
/// use alloy_core::primitives::{U256, Address};
///
/// // Store different types in slots
/// let counter: Slot<U256> = Slot::default();
/// let owner: Slot<Address> = Slot::default();
/// let active: Slot<bool> = Slot::default();
///
/// // Write values
/// counter.write(U256::from(100));
/// owner.write(Address::ZERO);
/// active.write(true);
/// ```
#[derive(Default)]
pub struct Slot<V> {
    /// The storage key for this slot
    id: U256,
    /// Phantom data to maintain type information
    _pd: PhantomData<V>,
}

impl<V> StorageLayout for Slot<V> {
    /// Creates a new slot with the specified storage key.
    ///
    /// This method is typically called by storage allocation macros to assign
    /// unique storage locations to contract state variables.
    ///
    /// # Arguments
    /// * `first` - Least significant 64 bits of the storage key
    /// * `second` - Second 64 bits of the storage key
    /// * `third` - Third 64 bits of the storage key
    /// * `fourth` - Most significant 64 bits of the storage key
    ///
    /// # Returns
    /// A new `Slot<V>` instance configured with the provided storage key
    fn allocate(first: u64, second: u64, third: u64, fourth: u64) -> Self {
        Self {
            id: U256::from_limbs([first, second, third, fourth]),
            _pd: PhantomData::default(),
        }
    }
}

impl<V> StorageStorable for Slot<V>
where
    V: SolValue + core::convert::From<<<V as SolValue>::SolType as SolType>::RustType>,
{
    type Value = V;

    /// Reads a value from storage at the specified key.
    ///
    /// This method performs an SLOAD operation to retrieve the raw bytes from storage,
    /// then ABI-decodes them into the target type. If decoding fails, the contract reverts.
    ///
    /// # Arguments
    /// * `key` - The storage key to read from
    ///
    /// # Returns
    /// The decoded value from storage
    ///
    /// # Panics
    /// Reverts the contract if ABI decoding fails, indicating corrupted storage data
    fn __read(key: U256) -> Self::Value {
        let bytes: [u8; 32] = sload(key).to_be_bytes();
        V::abi_decode(&bytes, false).unwrap_or_else(|_| revert())
    }

    /// Writes a value to storage at the specified key.
    ///
    /// This method ABI-encodes the value into bytes, pads it to 32 bytes if necessary,
    /// then performs an SSTORE operation to write it to storage.
    ///
    /// # Arguments
    /// * `key` - The storage key to write to
    /// * `value` - The value to encode and store
    fn __write(key: U256, value: Self::Value) {
        let bytes = value.abi_encode();
        let mut padded = [0u8; 32];
        padded[..bytes.len()].copy_from_slice(&bytes);
        sstore(key, U256::from_be_bytes(padded));
    }
}

impl<V> DirectStorage<V> for Slot<V>
where
    Self: StorageStorable<Value = V>,
{
    /// Reads the current value from this storage slot.
    ///
    /// # Returns
    /// The current value stored in this slot, decoded from storage
    ///
    /// # Examples
    /// ```rust,no_run
    /// let counter: Slot<U256> = Slot::default();
    /// let current_value = counter.read();
    /// ```
    fn read(&self) -> V {
        Self::__read(self.id)
    }

    /// Writes a new value to this storage slot.
    ///
    /// The value is ABI-encoded and written to the slot's storage location,
    /// replacing any previously stored value.
    ///
    /// # Arguments
    /// * `value` - The new value to store in this slot
    ///
    /// # Examples
    /// ```rust,no_run
    /// let mut counter: Slot<U256> = Slot::default();
    /// counter.write(U256::from(42));
    /// ```
    fn write(&mut self, value: V) {
        Self::__write(self.id, value)
    }
}

/// Arithmetic and comparison trait implementations to improve developer experience.
/// These traits allow slots to be used naturally in arithmetic expressions and comparisons.

/// Adds a value to the current slot value without modifying storage.
///
/// This operation reads the current value from storage, adds the right-hand side value,
/// and returns the result. The storage is not modified.
///
/// # Examples
/// ```rust,no_run
/// let counter: Slot<U256> = Slot::default();
/// counter.write(U256::from(10));
/// let result = counter + U256::from(5);  // Returns 15, storage unchanged
/// ```
impl<V> Add<V> for Slot<V>
where
    Self: StorageStorable<Value = V>,
    V: core::ops::Add<Output = V>,
{
    type Output = V;
    fn add(self, rhs: V) -> V {
        self.read() + rhs
    }
}

/// Adds a value to the current slot value and stores the result.
///
/// This operation reads the current value from storage, adds the right-hand side value,
/// and writes the result back to storage, updating the stored value.
///
/// # Examples
/// ```rust,no_run
/// let mut counter: Slot<U256> = Slot::default();
/// counter.write(U256::from(10));
/// counter += U256::from(5);  // Storage now contains 15
/// ```
impl<V> AddAssign<V> for Slot<V>
where
    Self: StorageStorable<Value = V>,
    V: core::ops::Add<Output = V>,
{
    fn add_assign(&mut self, rhs: V) {
        self.write(self.read() + rhs)
    }
}

/// Subtracts a value from the current slot value without modifying storage.
///
/// This operation reads the current value from storage, subtracts the right-hand side value,
/// and returns the result. The storage is not modified.
///
/// # Examples
/// ```rust,no_run
/// let counter: Slot<U256> = Slot::default();
/// counter.write(U256::from(10));
/// let result = counter - U256::from(3);  // Returns 7, storage unchanged
/// ```
impl<V> Sub<V> for Slot<V>
where
    Self: StorageStorable<Value = V>,
    V: core::ops::Sub<Output = V>,
{
    type Output = V;
    fn sub(self, rhs: V) -> V {
        self.read() - rhs
    }
}

/// Subtracts a value from the current slot value and stores the result.
///
/// This operation reads the current value from storage, subtracts the right-hand side value,
/// and writes the result back to storage, updating the stored value.
///
/// # Examples
/// ```rust,no_run
/// let mut counter: Slot<U256> = Slot::default();
/// counter.write(U256::from(10));
/// counter -= U256::from(3);  // Storage now contains 7
/// ```
impl<V> SubAssign<V> for Slot<V>
where
    Self: StorageStorable<Value = V>,
    V: core::ops::Sub<Output = V>,
{
    fn sub_assign(&mut self, rhs: V) {
        self.write(self.read() - rhs)
    }
}

/// Compares two slots for equality by reading their stored values.
///
/// This operation reads the values from both slots and compares them for equality.
/// Two slots are considered equal if they contain the same value, regardless of
/// their storage locations.
///
/// # Examples
/// ```rust,no_run
/// let slot1: Slot<U256> = Slot::default();
/// let slot2: Slot<U256> = Slot::default();
/// slot1.write(U256::from(42));
/// slot2.write(U256::from(42));
/// assert_eq!(slot1, slot2);  // true, both contain the same value
/// ```
impl<V> PartialEq for Slot<V>
where
    Self: StorageStorable<Value = V>,
    V: StorageStorable + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.read() == other.read()
    }
}
