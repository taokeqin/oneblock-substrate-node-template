// upgrade from v0 to v1
use frame_support::{migration::storage_key_iter, Blake2_128Concat};
use frame_support::{
	pallet_prelude::*, traits::GetStorageVersion, weights::Weight, StoragePrefixedMap,
};

use crate::{Config, Kitties, Kitty, KittyId, Pallet};
#[derive(Encode, Decode, Clone, Debug, TypeInfo, MaxEncodedLen, PartialEq, Eq)]
pub struct OldKitty {
	pub dna: [u8; 16],
	pub name: [u8; 4],
}

pub fn migrate<T: Config>() -> Weight {
	let on_chain_version = Pallet::<T>::on_chain_storage_version();
	let current_version = Pallet::<T>::current_storage_version();

	if on_chain_version != 1 {
		return Weight::zero();
	}

	if current_version != 2 {
		return Weight::zero();
	}

	let module = Kitties::<T>::module_prefix();
	let item = Kitties::<T>::storage_prefix();

	for (kitty_id, old_kitty) in
		storage_key_iter::<KittyId, OldKitty, Blake2_128Concat>(module, item).drain()
	{
		let mut new_kitty_name: [u8; 8] = [0; 8];
		for i in 0..old_kitty.name.len() {
			new_kitty_name[i] = old_kitty.name[i];
		}
		let new_kitty = Kitty { dna: old_kitty.dna, name: new_kitty_name };
		Kitties::<T>::insert(kitty_id, new_kitty);
	}
	Weight::zero()
}
