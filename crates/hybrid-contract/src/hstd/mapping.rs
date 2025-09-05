//! # Solidity-like Mapping Storage Implementation
//!
//! This module implements the `Mapping<K, V>` type, which provides Solidity-like mapping
//! functionality in the Hybrid VM environment. Mappings allow associating keys with values
//! in persistent storage, with automatic key hashing and type-safe access patterns.
//!
//! ## Overview
//!
//! A `Mapping<K, V>` simulates Solidity's mapping type by using keccak256 hashing to derive
//! storage keys from the mapping's base key and the provided lookup key. This ensures that
//! each key-value pair gets a unique storage location while maintaining compatibility with
//! Ethereum's storage model.
//!
//! ## Features
//!
//! - Type-safe key-value storage with automatic ABI encoding/decoding
//! - Keccak256-based key derivation matching Solidity's behavior
//! - Support for nested mappings (mapping of mappings)
//! - Guard-based access pattern for memory management
//! - Integration with the storage layout system
//!
//! ## Storage Key Derivation
//!
//! Storage keys for mapping values are computed as:
//! ```text
//! storage_key = keccak256(abi_encode(key) + abi_encode(mapping_id))
//! ```
//!
//! This matches Solidity's mapping storage key calculation, ensuring compatibility.
//!
//! ## Usage
//!
//! ```rust,no_run
//! use hybrid_contract::hstd::Mapping;
//! use alloy_core::primitives::{Address, U256};
//!
//! // Simple mapping
//! let balances: Mapping<Address, U256> = Mapping::default();
//! balances[user_address].write(U256::from(1000));
//! let balance = balances[user_address].read();
//!
//! // Nested mapping (mapping of mappings)
//! let allowances: Mapping<Address, Mapping<Address, U256>> = Mapping::default();
//! allowances[owner][spender].write(U256::from(500));
//! let allowance = allowances[owner][spender].read();
//! ```
//!
//! ## Memory Management
//!
//! Mappings use the global bump allocator to manage guard objects that provide access
//! to storage locations. Since the allocator never deallocates, guard objects remain
//! valid for the duration of the contract execution.

use core::{
    alloc::{GlobalAlloc, Layout},
    marker::PhantomData,
    ops::{Deref, DerefMut, Index, IndexMut},
};

use crate::allocator::ALLOC as GLOBAL;

use super::*;

/// A Solidity-like mapping type for key-value storage.
///
/// `Mapping<K, V>` provides persistent key-value storage that mimics Solidity's mapping
/// behavior. It uses keccak256 hashing to derive unique storage keys for each key-value
/// pair, ensuring efficient and collision-resistant storage access.
///
/// # Type Parameters
/// * `K` - The key type, must implement `SolValue` for ABI encoding
/// * `V` - The value type, can be any `StorageStorable` type including nested mappings
///
/// # Storage Layout
/// Each mapping instance has a unique base ID that is combined with individual keys
/// to produce storage locations. The actual values are accessed through guard objects
/// that manage the computed storage keys.
///
/// # Examples
/// ```rust,no_run
/// use hybrid_contract::hstd::Mapping;
/// use alloy_core::primitives::{Address, U256};
///
/// // User balance mapping
/// let balances: Mapping<Address, U256> = Mapping::default();
/// let user = Address::ZERO;
///
/// // Write a balance
/// balances[user].write(U256::from(1000));
///
/// // Read the balance
/// let balance = balances[user].read();
/// assert_eq!(balance, U256::from(1000));
/// ```
#[derive(Default)]
pub struct Mapping<K, V> {
    /// The unique identifier for this mapping instance
    id: U256,
    /// Phantom data to maintain type information about key and value types
    _pd: PhantomData<(K, V)>,
}

impl<K, V> StorageLayout for Mapping<K, V> {
    /// Creates a new mapping with the specified storage ID.
    ///
    /// This method is typically called by storage allocation macros to assign
    /// unique base IDs to mapping instances in the contract's storage layout.
    ///
    /// # Arguments
    /// * `first` - Least significant 64 bits of the mapping ID
    /// * `second` - Second 64 bits of the mapping ID
    /// * `third` - Third 64 bits of the mapping ID
    /// * `fourth` - Most significant 64 bits of the mapping ID
    ///
    /// # Returns
    /// A new `Mapping<K, V>` instance with the provided base ID
    fn allocate(first: u64, second: u64, third: u64, fourth: u64) -> Self {
        Self {
            id: U256::from_limbs([first, second, third, fourth]),
            _pd: PhantomData::default(),
        }
    }
}

impl<K, V> Mapping<K, V>
where
    K: SolValue,
{
    /// Computes the storage key for a given mapping key.
    ///
    /// This method implements Solidity's mapping key derivation algorithm:
    /// 1. ABI-encode the lookup key
    /// 2. ABI-encode the mapping's base ID
    /// 3. Concatenate the encoded values
    /// 4. Compute keccak256 of the concatenated data
    ///
    /// This ensures that each key-value pair gets a unique storage location
    /// while maintaining compatibility with Solidity's storage layout.
    ///
    /// # Arguments
    /// * `key` - The key to encode and hash
    ///
    /// # Returns
    /// The computed storage key as a U256
    ///
    /// # Examples
    /// ```rust,no_run
    /// // Internal usage - typically not called directly
    /// let storage_key = mapping.encode_key(user_address);
    /// ```
    fn encode_key(&self, key: K) -> U256 {
        let key_bytes = key.abi_encode();
        let id_bytes: [u8; 32] = self.id.to_be_bytes();

        // Concatenate the key bytes and id bytes
        let mut concatenated = Vec::with_capacity(key_bytes.len() + id_bytes.len());
        concatenated.extend_from_slice(&key_bytes);
        concatenated.extend_from_slice(&id_bytes);

        // Call the keccak256 syscall with the concatenated bytes
        let offset = concatenated.as_ptr() as u64;
        let size = concatenated.len() as u64;

        keccak256(offset, size)
    }
}

/// A guard object that provides access to a specific mapping storage location.
///
/// `MappingGuard<V>` is returned when indexing into a `Mapping` and serves as an
/// intermediate object that manages access to the computed storage location. It
/// provides methods to read from and write to the specific storage slot associated
/// with the mapping key.
///
/// # Type Parameters
/// * `V` - The value type stored in the mapping, must implement `StorageStorable`
///
/// # Lifetime
/// Guards are allocated using the global bump allocator and remain valid for the
/// duration of the contract execution. They do not need to be manually deallocated.
///
/// # Usage
/// Guards are typically not created directly but are returned by mapping indexing:
/// ```rust,no_run
/// let balances: Mapping<Address, U256> = Mapping::default();
/// let guard = &balances[user_address];  // Returns a MappingGuard
/// guard.write(U256::from(1000));
/// let balance = guard.read();
/// ```
pub struct MappingGuard<V>
where
    V: StorageStorable,
    V::Value:
        SolValue + core::convert::From<<<V::Value as SolValue>::SolType as SolType>::RustType>,
{
    /// The computed storage key for this mapping entry
    storage_key: U256,
    /// Phantom data to maintain type information about the value type
    _phantom: PhantomData<V>,
}

impl<V> MappingGuard<V>
where
    V: StorageStorable,
    V::Value:
        SolValue + core::convert::From<<<V::Value as SolValue>::SolType as SolType>::RustType>,
{
    /// Creates a new mapping guard for the specified storage key.
    ///
    /// # Arguments
    /// * `storage_key` - The storage key where the value is stored
    ///
    /// # Returns
    /// A new `MappingGuard<V>` that manages access to the storage location
    pub fn new(storage_key: U256) -> Self {
        Self {
            storage_key,
            _phantom: PhantomData,
        }
    }
}

impl<V> IndirectStorage<V> for MappingGuard<V>
where
    V: StorageStorable,
    V::Value:
        SolValue + core::convert::From<<<V::Value as SolValue>::SolType as SolType>::RustType>,
{
    /// Writes a value to storage at the location managed by this guard.
    ///
    /// This method performs an SSTORE operation to write the ABI-encoded value
    /// to the storage location computed for this mapping entry.
    ///
    /// # Arguments
    /// * `value` - The value to store at this mapping location
    ///
    /// # Examples
    /// ```rust,no_run
    /// let balances: Mapping<Address, U256> = Mapping::default();
    /// balances[user_address].write(U256::from(1000));
    /// ```
    fn write(&mut self, value: V::Value) {
        V::__write(self.storage_key, value);
    }

    /// Reads the value from storage at the location managed by this guard.
    ///
    /// This method performs an SLOAD operation to read and decode the value
    /// from the storage location computed for this mapping entry.
    ///
    /// # Returns
    /// The current value stored at this mapping location
    ///
    /// # Examples
    /// ```rust,no_run
    /// let balances: Mapping<Address, U256> = Mapping::default();
    /// let balance = balances[user_address].read();
    /// ```
    fn read(&self) -> V::Value {
        V::__read(self.storage_key)
    }
}

/// Index implementation for read-only access to mapping values.
///
/// This implementation allows immutable access to mapping values using the index syntax.
/// It returns a reference to a `MappingGuard` that can be used to read the value at
/// the specified key.
impl<K, V> Index<K> for Mapping<K, V>
where
    K: SolValue + 'static,
    V: StorageStorable + 'static,
    V::Value: SolValue
        + core::convert::From<<<V::Value as SolValue>::SolType as SolType>::RustType>
        + 'static,
{
    type Output = MappingGuard<V>;

    /// Provides read-only access to a mapping value by key.
    ///
    /// This method computes the storage key for the given mapping key, creates
    /// a guard object to manage access to that storage location, and returns
    /// a reference to the guard.
    ///
    /// # Arguments
    /// * `key` - The key to look up in the mapping
    ///
    /// # Returns
    /// A reference to a `MappingGuard<V>` that can read the value
    ///
    /// # Memory Management
    /// The guard is allocated using the global bump allocator and remains valid
    /// for the duration of the contract execution.
    ///
    /// # Examples
    /// ```rust,no_run
    /// let balances: Mapping<Address, U256> = Mapping::default();
    /// let balance = balances[user_address].read();  // Read-only access
    /// ```
    fn index(&self, key: K) -> &Self::Output {
        let storage_key = self.encode_key(key);

        // Create the guard
        let guard = MappingGuard::<V>::new(storage_key);

        // Manually handle memory using the global allocator
        unsafe {
            // Calculate layout for the guard which holds the mapping key
            let layout = Layout::new::<MappingGuard<V>>();

            // Allocate using the `GLOBAL` fixed memory allocator
            #[allow(static_mut_refs)]
            let ptr = GLOBAL.alloc(layout) as *mut MappingGuard<V>;

            // Write the guard to the allocated memory
            ptr.write(guard);

            // Return a reference with 'static lifetime (`GLOBAL` never deallocates)
            &*ptr
        }
    }
}

/// Mutable index implementation for read-write access to mapping values.
///
/// This implementation allows mutable access to mapping values using the index syntax.
/// It returns a mutable reference to a `MappingGuard` that can be used to both read
/// from and write to the value at the specified key.
impl<K, V> IndexMut<K> for Mapping<K, V>
where
    K: SolValue + 'static,
    V: StorageStorable + 'static,
    V::Value: SolValue
        + core::convert::From<<<V::Value as SolValue>::SolType as SolType>::RustType>
        + 'static,
{
    /// Provides mutable access to a mapping value by key.
    ///
    /// This method computes the storage key for the given mapping key, creates
    /// a guard object to manage access to that storage location, and returns
    /// a mutable reference to the guard.
    ///
    /// # Arguments
    /// * `key` - The key to look up in the mapping
    ///
    /// # Returns
    /// A mutable reference to a `MappingGuard<V>` that can read and write the value
    ///
    /// # Memory Management
    /// The guard is allocated using the global bump allocator and remains valid
    /// for the duration of the contract execution.
    ///
    /// # Examples
    /// ```rust,no_run
    /// let mut balances: Mapping<Address, U256> = Mapping::default();
    /// balances[user_address].write(U256::from(1000));  // Read-write access
    /// ```
    fn index_mut(&mut self, key: K) -> &mut Self::Output {
        let storage_key = self.encode_key(key);

        // Create the guard
        let guard = MappingGuard::<V>::new(storage_key);

        // Manually handle memory using the global allocator
        unsafe {
            // Calculate layout for the guard which holds the mapping key
            let layout = Layout::new::<MappingGuard<V>>();

            // Allocate using the `GLOBAL` fixed memory allocator
            #[allow(static_mut_refs)]
            let ptr = GLOBAL.alloc(layout) as *mut MappingGuard<V>;

            // Write the guard to the allocated memory
            ptr.write(guard);

            // Return a reference with 'static lifetime (`GLOBAL` never deallocates)
            &mut *ptr
        }
    }
}

/// A wrapper for nested mappings that provides proper `Deref` behavior.
///
/// `NestedMapping<K2, V>` wraps a `Mapping<K2, V>` to enable nested mapping access
/// patterns. This is used internally when accessing mappings that contain other
/// mappings as their values (e.g., `Mapping<Address, Mapping<Address, U256>>`).
///
/// # Type Parameters
/// * `K2` - The inner mapping's key type
/// * `V` - The final value type stored in the nested mapping
///
/// # Usage
/// This type is typically not used directly but is returned when indexing into
/// nested mappings:
/// ```rust,no_run
/// let allowances: Mapping<Address, Mapping<Address, U256>> = Mapping::default();
/// let inner = &allowances[owner];  // Returns NestedMapping<Address, U256>
/// inner[spender].write(U256::from(500));
/// ```
pub struct NestedMapping<K2, V> {
    /// The inner mapping instance
    mapping: Mapping<K2, V>,
}

impl<K2, V> Deref for NestedMapping<K2, V> {
    type Target = Mapping<K2, V>;

    /// Provides immutable access to the inner mapping.
    ///
    /// This allows `NestedMapping` to be used transparently as a `Mapping`
    /// for read operations.
    fn deref(&self) -> &Self::Target {
        &self.mapping
    }
}

impl<K2, V> DerefMut for NestedMapping<K2, V> {
    /// Provides mutable access to the inner mapping.
    ///
    /// This allows `NestedMapping` to be used transparently as a `Mapping`
    /// for both read and write operations.
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.mapping
    }
}

/// Index implementation for nested mappings.
impl<K1, K2, V> Index<K1> for Mapping<K1, Mapping<K2, V>>
where
    K1: SolValue + 'static,
    K2: SolValue + 'static,
    V: 'static,
{
    type Output = NestedMapping<K2, V>;

    /// Provides read-only access to a nested mapping by outer key.
    ///
    /// This method handles the first level of indexing for nested mappings by:
    /// 1. Computing a storage key from the outer mapping key
    /// 2. Creating an inner mapping with that derived key as its base ID
    /// 3. Wrapping it in a `NestedMapping` for proper access patterns
    ///
    /// # Arguments
    /// * `key` - The outer mapping key (first level of nesting)
    ///
    /// # Returns
    /// A reference to a `NestedMapping<K2, V>` for accessing inner values
    ///
    /// # Examples
    /// ```rust,no_run
    /// let allowances: Mapping<Address, Mapping<Address, U256>> = Mapping::default();
    /// let owner_allowances = &allowances[owner_address];
    /// let allowance = owner_allowances[spender_address].read();
    /// ```
    fn index(&self, key: K1) -> &Self::Output {
        let id = self.encode_key(key);

        // Create the nested mapping
        let mapping = Mapping {
            id,
            _pd: PhantomData,
        };
        let nested = NestedMapping { mapping };

        // Manually handle memory using the global allocator
        unsafe {
            // Calculate layout for the nested mapping
            // which is an intermediate object that links to the inner-most mapping guard
            let layout = Layout::new::<NestedMapping<K2, V>>();

            // Allocate using the `GLOBAL` fixed memory allocator
            #[allow(static_mut_refs)]
            let ptr = GLOBAL.alloc(layout) as *mut NestedMapping<K2, V>;

            // Write the nested mapping to the allocated memory
            ptr.write(nested);

            // Return a reference with 'static lifetime (`GLOBAL` never deallocates)
            &*ptr
        }
    }
}

/// Index implementation for nested mappings.
impl<K1, K2, V> IndexMut<K1> for Mapping<K1, Mapping<K2, V>>
where
    K1: SolValue + 'static,
    K2: SolValue + 'static,
    V: 'static,
{
    /// Provides mutable access to a nested mapping by outer key.
    ///
    /// This method handles the first level of mutable indexing for nested mappings by:
    /// 1. Computing a storage key from the outer mapping key
    /// 2. Creating an inner mapping with that derived key as its base ID
    /// 3. Wrapping it in a `NestedMapping` for proper access patterns
    ///
    /// # Arguments
    /// * `key` - The outer mapping key (first level of nesting)
    ///
    /// # Returns
    /// A mutable reference to a `NestedMapping<K2, V>` for accessing inner values
    ///
    /// # Examples
    /// ```rust,no_run
    /// let mut allowances: Mapping<Address, Mapping<Address, U256>> = Mapping::default();
    /// allowances[owner_address][spender_address].write(U256::from(500));
    /// ```
    fn index_mut(&mut self, key: K1) -> &mut Self::Output {
        let id = self.encode_key(key);

        // Create the nested mapping
        let mapping = Mapping {
            id,
            _pd: PhantomData,
        };
        let nested = NestedMapping { mapping };

        // Manually handle memory using the global allocator
        unsafe {
            // Calculate layout for the nested mapping
            // which is an intermediate object that links to the inner-most mapping guard
            let layout = Layout::new::<NestedMapping<K2, V>>();

            // Allocate using the `GLOBAL` fixed memory allocator
            #[allow(static_mut_refs)]
            let ptr = GLOBAL.alloc(layout) as *mut NestedMapping<K2, V>;

            // Write the nested mapping to the allocated memory
            ptr.write(nested);

            // Return a reference with 'static lifetime (`GLOBAL` never deallocates)
            &mut *ptr
        }
    }
}
