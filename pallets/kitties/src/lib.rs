#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
mod migrations;
#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use crate::migrations;
	use frame_support::pallet_prelude::{ValueQuery, *};
	use frame_support::traits::{Currency, ExistenceRequirement, Randomness};
	use frame_support::PalletId;
	use frame_system::pallet_prelude::*;
	use sp_io::hashing::blake2_128;
	use sp_runtime::traits::AccountIdConversion;
	pub type KittyId = u32;
	pub type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
	#[derive(
		Encode, Decode, Clone, Copy, RuntimeDebug, PartialEq, Eq, Default, TypeInfo, MaxEncodedLen,
	)]
	//pub struct Kitty(pub [u8; 16]);
	pub struct Kitty {
		pub dna: [u8; 16],
		pub name: [u8; 8],
	}
	const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);
	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
		type Currency: Currency<Self::AccountId>;
		#[pallet::constant]
		type KittyPrice: Get<BalanceOf<Self>>;

		type PalletId: Get<PalletId>;
	}

	#[pallet::storage]
	#[pallet::getter(fn next_kitty_id)]
	pub type NextKittyId<T: Config> = StorageValue<_, KittyId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn kitties)]
	pub type Kitties<T: Config> = StorageMap<_, Blake2_128Concat, KittyId, Kitty>;

	#[pallet::storage]
	#[pallet::getter(fn owner)]
	pub type KittyOwner<T: Config> = StorageMap<_, Blake2_128Concat, KittyId, T::AccountId>;

	// Kitty parents
	#[pallet::storage]
	#[pallet::getter(fn kitty_parents)]
	pub type KittyParents<T: Config> =
		StorageMap<_, Blake2_128Concat, KittyId, (KittyId, KittyId), ValueQuery>;

	// kitty on sale
	#[pallet::storage]
	#[pallet::getter(fn kitty_on_sale)]
	pub type KittyOnSale<T: Config> = StorageMap<_, Blake2_128Concat, KittyId, ()>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		KittyCreated { who: T::AccountId, kitty_id: KittyId, kitty: Kitty },
		KittyBreed { who: T::AccountId, kitty_id: KittyId, kitty: Kitty },
		KittyTransferred { from: T::AccountId, to: T::AccountId, kitty_id: KittyId },
		KittyOnSale { who: T::AccountId, kitty_id: KittyId },
		KittyBought { who: T::AccountId, kitty_id: KittyId },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		InvalidKittyId,
		AlreadyOnSale,
		NotOnSale,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_runtime_upgrade() -> frame_support::weights::Weight {
			// do something on runtime upgrade
			migrations::migrate::migrate::<T>()
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(10_1000)]
		pub fn create(origin: OriginFor<T>, name: [u8; 8]) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let kitty_id = Self::get_next_id()?;
			ensure!(kitty_id != KittyId::max_value(), Error::<T>::StorageOverflow);
			let dna = Self::random_value(&who);
			let kitty = Kitty { dna, name };
			let price = T::KittyPrice::get();
			//T::Currency::reserve(&who, price)?;
			T::Currency::transfer(
				&who,
				&Self::get_account_id(),
				price,
				ExistenceRequirement::KeepAlive,
			)?;
			Kitties::<T>::insert(kitty_id, kitty.clone());
			KittyOwner::<T>::insert(kitty_id, who.clone());
			// Emit an event.
			Self::deposit_event(Event::KittyCreated { who, kitty_id, kitty });
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		//breed kitty from two parents
		#[pallet::call_index(1)]
		#[pallet::weight(10_1000)]
		pub fn breed(
			origin: OriginFor<T>,
			kitty_id_1: KittyId,
			kitty_id_2: KittyId,
			name: [u8; 8],
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(Kitties::<T>::contains_key(kitty_id_1), Error::<T>::InvalidKittyId);
			ensure!(Kitties::<T>::contains_key(kitty_id_2), Error::<T>::InvalidKittyId);
			let kitty_id = Self::get_next_id()?;
			ensure!(kitty_id != KittyId::max_value(), Error::<T>::StorageOverflow);

			let base_data = Self::random_value(&who);

			let kitty1 = Kitties::<T>::get(kitty_id_1).ok_or(Error::<T>::InvalidKittyId)?;
			let kitty2 = Kitties::<T>::get(kitty_id_2).ok_or(Error::<T>::InvalidKittyId)?;

			let mut new_kitty_data = [0u8; 16];
			for i in 0..kitty1.dna.len() {
				new_kitty_data[i] =
					(kitty1.dna[i] & base_data[i]) | (!kitty2.dna[i] & !base_data[i]);
			}
			//let dna = Self::random_value(&who); // TODO: fix this with real dna
			let kitty = Kitty { dna: new_kitty_data, name };
			let price = T::KittyPrice::get();
			//T::Currency::reserve(&who, price)?;

			Kitties::<T>::insert(kitty_id, kitty.clone());
			KittyOwner::<T>::insert(kitty_id, who.clone());
			KittyParents::<T>::insert(kitty_id, (kitty_id_1, kitty_id_2));
			// Emit an event.
			Self::deposit_event(Event::KittyBreed { who, kitty_id, kitty });
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		// transfer kitty to another account
		#[pallet::call_index(2)]
		#[pallet::weight(10_1000)]
		pub fn transfer(
			origin: OriginFor<T>,
			to: T::AccountId,
			kitty_id: KittyId,
		) -> DispatchResult {
			let from = ensure_signed(origin)?;
			ensure!(Kitties::<T>::contains_key(kitty_id), Error::<T>::InvalidKittyId);
			ensure!(KittyOwner::<T>::contains_key(kitty_id), Error::<T>::InvalidKittyId);

			let owner = KittyOwner::<T>::get(kitty_id).ok_or(Error::<T>::InvalidKittyId)?;
			ensure!(owner == from, Error::<T>::InvalidKittyId);

			KittyOwner::<T>::insert(kitty_id, to.clone());

			// Emit an event.
			Self::deposit_event(Event::KittyTransferred { from, to, kitty_id });
			Ok(())
		}

		// sell kitty
		#[pallet::call_index(3)]
		#[pallet::weight(10_1000)]
		pub fn sale(origin: OriginFor<T>, kitty_id: KittyId) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(Kitties::<T>::contains_key(kitty_id), Error::<T>::InvalidKittyId);
			ensure!(KittyOwner::<T>::contains_key(kitty_id), Error::<T>::InvalidKittyId);

			let owner = KittyOwner::<T>::get(kitty_id).ok_or(Error::<T>::InvalidKittyId)?;
			ensure!(owner == who, Error::<T>::InvalidKittyId);
			ensure!(Self::kitty_on_sale(kitty_id).is_some(), Error::<T>::AlreadyOnSale);

			KittyOnSale::<T>::insert(kitty_id, ());
			Self::deposit_event(Event::KittyOnSale { who, kitty_id });
			Ok(())
		}

		// buy kitty
		#[pallet::call_index(4)]
		#[pallet::weight(10_1000)]
		pub fn buy(origin: OriginFor<T>, kitty_id: KittyId) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(Kitties::<T>::contains_key(kitty_id), Error::<T>::InvalidKittyId);
			ensure!(KittyOwner::<T>::contains_key(kitty_id), Error::<T>::InvalidKittyId);

			let owner = KittyOwner::<T>::get(kitty_id).ok_or(Error::<T>::InvalidKittyId)?;
			ensure!(owner != who, Error::<T>::InvalidKittyId);
			ensure!(Self::kitty_on_sale(kitty_id).is_some(), Error::<T>::NotOnSale);

			let price = T::KittyPrice::get();
			//T::Currency::reserve(&who, price)?;
			//T::Currency::unreserve(&owner, price);
			T::Currency::transfer(&who, &owner, price, ExistenceRequirement::KeepAlive)?;
			KittyOwner::<T>::insert(kitty_id, who.clone());
			KittyOnSale::<T>::remove(kitty_id);

			Self::deposit_event(Event::KittyBought { who, kitty_id });
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn get_next_id() -> Result<KittyId, DispatchError> {
			NextKittyId::<T>::try_mutate(|next_id| -> Result<KittyId, DispatchError> {
				let current_id = *next_id;
				*next_id = next_id
					.checked_add(1)
					.ok_or::<DispatchError>(Error::<T>::InvalidKittyId.into())?;
				Ok(current_id)
			})
		}

		fn random_value(sender: &T::AccountId) -> [u8; 16] {
			let payload = (
				T::Randomness::random_seed(),
				&sender,
				<frame_system::Pallet<T>>::extrinsic_index(),
			);
			payload.using_encoded(blake2_128)
		}
		fn get_account_id() -> T::AccountId {
			T::PalletId::get().into_account_truncating()
		}
	}
}
