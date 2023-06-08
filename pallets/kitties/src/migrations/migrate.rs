use crate::{Config, Pallet};
use frame_support::{pallet_prelude::*, traits::GetStorageVersion, weights::Weight};

//use crate::migrations::v0_to_v1;
use crate::migrations::v0_to_v2;
use crate::migrations::v1_to_v2;

pub fn migrate<T: Config>() -> Weight {
	let on_chain_version = Pallet::<T>::on_chain_storage_version();
	let current_version = Pallet::<T>::current_storage_version();

	if on_chain_version == 0 && current_version == 1 {
		//v0_to_v1::migrate::<T>();
		// do nothing for this version.
	}

	if on_chain_version == 0 && current_version == 2 {
		v0_to_v2::migrate::<T>();
	}

	if on_chain_version == 1 && current_version == 2 {
		v1_to_v2::migrate::<T>();
	}

	Weight::zero()
}
