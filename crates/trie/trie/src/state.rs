use crate::{
<<<<<<< HEAD
    hashed_cursor::HashedPostStateCursorFactory,
    prefix_set::{PrefixSetMut, TriePrefixSetsMut},
    proof::Proof,
    updates::TrieUpdates,
    Nibbles, StateRoot,
=======
    prefix_set::{PrefixSetMut, TriePrefixSetsMut},
    Nibbles,
>>>>>>> c4b5f5e9c9a88783b2def3ab1cc880b8d41867e1
};
use itertools::Itertools;
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
<<<<<<< HEAD
use reth_db::{tables, DatabaseError};
use reth_db_api::{
    cursor::DbCursorRO,
    models::{AccountBeforeTx, BlockNumberAddress},
    transaction::DbTx,
};
use reth_execution_errors::StateRootError;
use reth_primitives::{keccak256, Account, Address, BlockNumber, B256, U256};
use reth_trie_common::AccountProof;
use revm::db::BundleAccount;
use std::{
    collections::{hash_map, HashMap, HashSet},
    ops::RangeInclusive,
};
=======
use reth_primitives::{keccak256, Account, Address, B256, U256};
use revm::db::{states::StorageSlot, AccountStatus, BundleAccount};
use std::collections::{hash_map, HashMap, HashSet};
>>>>>>> c4b5f5e9c9a88783b2def3ab1cc880b8d41867e1

/// Representation of in-memory hashed state.
#[derive(PartialEq, Eq, Clone, Default, Debug)]
pub struct HashedPostState {
    /// Mapping of hashed address to account info, `None` if destroyed.
    pub accounts: HashMap<B256, Option<Account>>,
    /// Mapping of hashed address to hashed storage.
    pub storages: HashMap<B256, HashedStorage>,
}

impl HashedPostState {
    /// Initialize [`HashedPostState`] from bundle state.
    /// Hashes all changed accounts and storage entries that are currently stored in the bundle
    /// state.
    pub fn from_bundle_state<'a>(
        state: impl IntoParallelIterator<Item = (&'a Address, &'a BundleAccount)>,
    ) -> Self {
        let hashed = state
            .into_par_iter()
            .map(|(address, account)| {
                let hashed_address = keccak256(address);
                let hashed_account = account.info.clone().map(Into::into);
<<<<<<< HEAD
                let hashed_storage = HashedStorage::from_iter(
                    account.status.was_destroyed(),
                    account.storage.iter().map(|(key, value)| {
                        (keccak256(B256::new(key.to_be_bytes())), value.present_value)
                    }),
                );
=======
                let hashed_storage =
                    HashedStorage::from_bundle_state(account.status, &account.storage);
>>>>>>> c4b5f5e9c9a88783b2def3ab1cc880b8d41867e1
                (hashed_address, (hashed_account, hashed_storage))
            })
            .collect::<Vec<(B256, (Option<Account>, HashedStorage))>>();

        let mut accounts = HashMap::with_capacity(hashed.len());
        let mut storages = HashMap::with_capacity(hashed.len());
        for (address, (account, storage)) in hashed {
            accounts.insert(address, account);
            storages.insert(address, storage);
        }
        Self { accounts, storages }
    }

    /// Construct [`HashedPostState`] from a single [`HashedStorage`].
    pub fn from_hashed_storage(hashed_address: B256, storage: HashedStorage) -> Self {
        Self { accounts: HashMap::default(), storages: HashMap::from([(hashed_address, storage)]) }
    }

    /// Set account entries on hashed state.
    pub fn with_accounts(
        mut self,
        accounts: impl IntoIterator<Item = (B256, Option<Account>)>,
    ) -> Self {
        self.accounts = HashMap::from_iter(accounts);
        self
    }

    /// Set storage entries on hashed state.
    pub fn with_storages(
        mut self,
        storages: impl IntoIterator<Item = (B256, HashedStorage)>,
    ) -> Self {
        self.storages = HashMap::from_iter(storages);
        self
    }

    /// Extend this hashed post state with contents of another.
    /// Entries in the second hashed post state take precedence.
    pub fn extend(&mut self, other: Self) {
        for (hashed_address, account) in other.accounts {
            self.accounts.insert(hashed_address, account);
        }

        for (hashed_address, storage) in other.storages {
            match self.storages.entry(hashed_address) {
                hash_map::Entry::Vacant(entry) => {
                    entry.insert(storage);
                }
                hash_map::Entry::Occupied(mut entry) => {
                    entry.get_mut().extend(storage);
                }
            }
        }
    }

    /// Converts hashed post state into [`HashedPostStateSorted`].
    pub fn into_sorted(self) -> HashedPostStateSorted {
        let mut updated_accounts = Vec::new();
        let mut destroyed_accounts = HashSet::default();
        for (hashed_address, info) in self.accounts {
            if let Some(info) = info {
                updated_accounts.push((hashed_address, info));
            } else {
                destroyed_accounts.insert(hashed_address);
            }
        }
        updated_accounts.sort_unstable_by_key(|(address, _)| *address);
        let accounts = HashedAccountsSorted { accounts: updated_accounts, destroyed_accounts };

        let storages = self
            .storages
            .into_iter()
            .map(|(hashed_address, storage)| (hashed_address, storage.into_sorted()))
            .collect();

        HashedPostStateSorted { accounts, storages }
    }

    /// Construct [`TriePrefixSetsMut`] from hashed post state.
    /// The prefix sets contain the hashed account and storage keys that have been changed in the
    /// post state.
    pub fn construct_prefix_sets(&self) -> TriePrefixSetsMut {
        // Populate account prefix set.
        let mut account_prefix_set = PrefixSetMut::with_capacity(self.accounts.len());
        let mut destroyed_accounts = HashSet::default();
        for (hashed_address, account) in &self.accounts {
            account_prefix_set.insert(Nibbles::unpack(hashed_address));

            if account.is_none() {
                destroyed_accounts.insert(*hashed_address);
            }
        }

        // Populate storage prefix sets.
        let mut storage_prefix_sets = HashMap::with_capacity(self.storages.len());
        for (hashed_address, hashed_storage) in &self.storages {
            account_prefix_set.insert(Nibbles::unpack(hashed_address));
<<<<<<< HEAD

            let mut prefix_set = PrefixSetMut::with_capacity(hashed_storage.storage.len());
            for hashed_slot in hashed_storage.storage.keys() {
                prefix_set.insert(Nibbles::unpack(hashed_slot));
            }
            storage_prefix_sets.insert(*hashed_address, prefix_set);
        }

        TriePrefixSetsMut { account_prefix_set, storage_prefix_sets, destroyed_accounts }
    }

    /// Calculate the state root for this [`HashedPostState`].
    /// Internally, this method retrieves prefixsets and uses them
    /// to calculate incremental state root.
    ///
    /// # Example
    ///
    /// ```
    /// use reth_db::test_utils::create_test_rw_db;
    /// use reth_db_api::database::Database;
    /// use reth_primitives::{Account, U256};
    /// use reth_trie::HashedPostState;
    ///
    /// // Initialize the database
    /// let db = create_test_rw_db();
    ///
    /// // Initialize hashed post state
    /// let mut hashed_state = HashedPostState::default();
    /// hashed_state.accounts.insert(
    ///     [0x11; 32].into(),
    ///     Some(Account { nonce: 1, balance: U256::from(10), bytecode_hash: None }),
    /// );
    ///
    /// // Calculate the state root
    /// let tx = db.tx().expect("failed to create transaction");
    /// let state_root = hashed_state.state_root(&tx);
    /// ```
    ///
    /// # Returns
    ///
    /// The state root for this [`HashedPostState`].
    pub fn state_root<TX: DbTx>(&self, tx: &TX) -> Result<B256, StateRootError> {
        let sorted = self.clone().into_sorted();
        let prefix_sets = self.construct_prefix_sets().freeze();
        StateRoot::from_tx(tx)
            .with_hashed_cursor_factory(HashedPostStateCursorFactory::new(tx, &sorted))
            .with_prefix_sets(prefix_sets)
            .root()
    }

    /// Calculates the state root for this [`HashedPostState`] and returns it alongside trie
    /// updates. See [`Self::state_root`] for more info.
    pub fn state_root_with_updates<TX: DbTx>(
        &self,
        tx: &TX,
    ) -> Result<(B256, TrieUpdates), StateRootError> {
        let sorted = self.clone().into_sorted();
        let prefix_sets = self.construct_prefix_sets().freeze();
        StateRoot::from_tx(tx)
            .with_hashed_cursor_factory(HashedPostStateCursorFactory::new(tx, &sorted))
            .with_prefix_sets(prefix_sets)
            .root_with_updates()
=======
            storage_prefix_sets.insert(*hashed_address, hashed_storage.construct_prefix_set());
        }

        TriePrefixSetsMut { account_prefix_set, storage_prefix_sets, destroyed_accounts }
>>>>>>> c4b5f5e9c9a88783b2def3ab1cc880b8d41867e1
    }

    /// Generates the state proof for target account and slots on top of this [`HashedPostState`].
    pub fn account_proof<TX: DbTx>(
        &self,
        tx: &TX,
        address: Address,
        slots: &[B256],
    ) -> Result<AccountProof, StateRootError> {
        let sorted = self.clone().into_sorted();
        let prefix_sets = self.construct_prefix_sets();
        Proof::from_tx(tx)
            .with_hashed_cursor_factory(HashedPostStateCursorFactory::new(tx, &sorted))
            .with_prefix_sets_mut(prefix_sets)
            .account_proof(address, slots)
    }
}

/// Representation of in-memory hashed storage.
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct HashedStorage {
    /// Flag indicating whether the storage was wiped or not.
    pub wiped: bool,
    /// Mapping of hashed storage slot to storage value.
    pub storage: HashMap<B256, U256>,
}

impl HashedStorage {
    /// Create new instance of [`HashedStorage`].
    pub fn new(wiped: bool) -> Self {
        Self { wiped, storage: HashMap::default() }
    }

    /// Create new hashed storage from iterator.
    pub fn from_iter(wiped: bool, iter: impl IntoIterator<Item = (B256, U256)>) -> Self {
        Self { wiped, storage: HashMap::from_iter(iter) }
    }

    /// Create new hashed storage from bundle state account entry.
    pub fn from_bundle_state(status: AccountStatus, storage: &HashMap<U256, StorageSlot>) -> Self {
        let storage = storage
            .iter()
            .map(|(key, value)| (keccak256(B256::from(*key)), value.present_value))
            .collect();
        Self { wiped: status.was_destroyed(), storage }
    }

    /// Construct [`PrefixSetMut`] from hashed storage.
    pub fn construct_prefix_set(&self) -> PrefixSetMut {
        let mut prefix_set = PrefixSetMut::with_capacity(self.storage.len());
        for hashed_slot in self.storage.keys() {
            prefix_set.insert(Nibbles::unpack(hashed_slot));
        }
        prefix_set
    }

    /// Extend hashed storage with contents of other.
    /// The entries in second hashed storage take precedence.
    pub fn extend(&mut self, other: Self) {
        if other.wiped {
            self.wiped = true;
            self.storage.clear();
        }
        for (hashed_slot, value) in other.storage {
            self.storage.insert(hashed_slot, value);
        }
    }

    /// Converts hashed storage into [`HashedStorageSorted`].
    pub fn into_sorted(self) -> HashedStorageSorted {
        let mut non_zero_valued_slots = Vec::new();
        let mut zero_valued_slots = HashSet::default();
        for (hashed_slot, value) in self.storage {
            if value.is_zero() {
                zero_valued_slots.insert(hashed_slot);
            } else {
                non_zero_valued_slots.push((hashed_slot, value));
            }
        }
        non_zero_valued_slots.sort_unstable_by_key(|(key, _)| *key);

        HashedStorageSorted { non_zero_valued_slots, zero_valued_slots, wiped: self.wiped }
    }
}

/// Sorted hashed post state optimized for iterating during state trie calculation.
#[derive(PartialEq, Eq, Clone, Default, Debug)]
pub struct HashedPostStateSorted {
    /// Updated state of accounts.
    pub(crate) accounts: HashedAccountsSorted,
    /// Map of hashed addresses to hashed storage.
    pub(crate) storages: HashMap<B256, HashedStorageSorted>,
}

<<<<<<< HEAD
/// Sorted account state optimized for iterating during state trie calculation.
#[derive(Clone, Eq, PartialEq, Debug)]
=======
impl HashedPostStateSorted {
    /// Create new instance of [`HashedPostStateSorted`]
    pub const fn new(
        accounts: HashedAccountsSorted,
        storages: HashMap<B256, HashedStorageSorted>,
    ) -> Self {
        Self { accounts, storages }
    }

    /// Returns reference to hashed accounts.
    pub const fn accounts(&self) -> &HashedAccountsSorted {
        &self.accounts
    }

    /// Returns reference to hashed account storages.
    pub const fn account_storages(&self) -> &HashMap<B256, HashedStorageSorted> {
        &self.storages
    }
}

/// Sorted account state optimized for iterating during state trie calculation.
#[derive(Clone, Eq, PartialEq, Default, Debug)]
>>>>>>> c4b5f5e9c9a88783b2def3ab1cc880b8d41867e1
pub struct HashedAccountsSorted {
    /// Sorted collection of hashed addresses and their account info.
    pub(crate) accounts: Vec<(B256, Account)>,
    /// Set of destroyed account keys.
    pub(crate) destroyed_accounts: HashSet<B256>,
<<<<<<< HEAD
=======
}

impl HashedAccountsSorted {
    /// Returns a sorted iterator over updated accounts.
    pub fn accounts_sorted(&self) -> impl Iterator<Item = (B256, Option<Account>)> {
        self.accounts
            .iter()
            .map(|(address, account)| (*address, Some(*account)))
            .chain(self.destroyed_accounts.iter().map(|address| (*address, None)))
            .sorted_by_key(|entry| *entry.0)
    }
>>>>>>> c4b5f5e9c9a88783b2def3ab1cc880b8d41867e1
}

/// Sorted hashed storage optimized for iterating during state trie calculation.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct HashedStorageSorted {
    /// Sorted hashed storage slots with non-zero value.
    pub(crate) non_zero_valued_slots: Vec<(B256, U256)>,
    /// Slots that have been zero valued.
    pub(crate) zero_valued_slots: HashSet<B256>,
    /// Flag indicating whether the storage was wiped or not.
    pub(crate) wiped: bool,
}

impl HashedStorageSorted {
    /// Returns `true` if the account was wiped.
    pub const fn is_wiped(&self) -> bool {
        self.wiped
    }

    /// Returns a sorted iterator over updated storage slots.
    pub fn storage_slots_sorted(&self) -> impl Iterator<Item = (B256, U256)> {
        self.non_zero_valued_slots
            .iter()
            .map(|(hashed_slot, value)| (*hashed_slot, *value))
            .chain(self.zero_valued_slots.iter().map(|hashed_slot| (*hashed_slot, U256::ZERO)))
            .sorted_by_key(|entry| *entry.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hashed_state_wiped_extension() {
        let hashed_address = B256::default();
        let hashed_slot = B256::with_last_byte(64);
        let hashed_slot2 = B256::with_last_byte(65);

        // Initialize post state storage
        let original_slot_value = U256::from(123);
        let mut hashed_state = HashedPostState::default().with_storages([(
            hashed_address,
            HashedStorage::from_iter(
                false,
                [(hashed_slot, original_slot_value), (hashed_slot2, original_slot_value)],
            ),
        )]);

        // Update single slot value
        let updated_slot_value = U256::from(321);
        let extension = HashedPostState::default().with_storages([(
            hashed_address,
            HashedStorage::from_iter(false, [(hashed_slot, updated_slot_value)]),
        )]);
        hashed_state.extend(extension);

        let account_storage = hashed_state.storages.get(&hashed_address);
        assert_eq!(
            account_storage.and_then(|st| st.storage.get(&hashed_slot)),
            Some(&updated_slot_value)
        );
        assert_eq!(
            account_storage.and_then(|st| st.storage.get(&hashed_slot2)),
            Some(&original_slot_value)
        );
        assert_eq!(account_storage.map(|st| st.wiped), Some(false));

        // Wipe account storage
        let wiped_extension =
            HashedPostState::default().with_storages([(hashed_address, HashedStorage::new(true))]);
        hashed_state.extend(wiped_extension);

        let account_storage = hashed_state.storages.get(&hashed_address);
        assert_eq!(account_storage.map(|st| st.storage.is_empty()), Some(true));
        assert_eq!(account_storage.map(|st| st.wiped), Some(true));

        // Reinitialize single slot value
        hashed_state.extend(HashedPostState::default().with_storages([(
            hashed_address,
            HashedStorage::from_iter(false, [(hashed_slot, original_slot_value)]),
        )]));
        let account_storage = hashed_state.storages.get(&hashed_address);
        assert_eq!(
            account_storage.and_then(|st| st.storage.get(&hashed_slot)),
            Some(&original_slot_value)
        );
        assert_eq!(account_storage.and_then(|st| st.storage.get(&hashed_slot2)), None);
        assert_eq!(account_storage.map(|st| st.wiped), Some(true));

        // Reinitialize single slot value
        hashed_state.extend(HashedPostState::default().with_storages([(
            hashed_address,
            HashedStorage::from_iter(false, [(hashed_slot2, updated_slot_value)]),
        )]));
        let account_storage = hashed_state.storages.get(&hashed_address);
        assert_eq!(
            account_storage.and_then(|st| st.storage.get(&hashed_slot)),
            Some(&original_slot_value)
        );
        assert_eq!(
            account_storage.and_then(|st| st.storage.get(&hashed_slot2)),
            Some(&updated_slot_value)
        );
        assert_eq!(account_storage.map(|st| st.wiped), Some(true));
    }
}
