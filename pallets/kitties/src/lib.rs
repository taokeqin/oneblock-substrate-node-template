#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_support::traits::Randomness;
	use frame_system::pallet_prelude::*;
	use sp_io::hashing::blake2_128;

	pub type KittyId = u32;

	#[derive(
		Encode, Decode, Clone, Copy, RuntimeDebug, PartialEq, Eq, Default, TypeInfo, MaxEncodedLen,
	)]
	pub struct Kitty(pub [u8; 16]);

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
	}

	#[pallet::storage]
	#[pallet::getter(fn next_kitty_id)]
	pub type NextKittyId<T> = StorageValue<_, KittyId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn kitties)]
	pub type Kitties<T> = StorageMap<_, Blake2_128Concat, KittyId, Kitty>;

	#[pallet::storage]
	#[pallet::getter(fn owner)]
	pub type KittyOwner<T: Config> = StorageMap<_, Blake2_128Concat, KittyId, T::AccountId>;

	// Kitty parents
	#[pallet::storage]
	#[pallet::getter(fn kitty_parents)]
	pub type KittyParents<T: Config> =
		StorageMap<_, Blake2_128Concat, KittyId, (KittyId, KittyId), ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		KittyCreated { who: T::AccountId, kitty_id: KittyId, kitty: Kitty },
		KittyBreed { who: T::AccountId, kitty_id: KittyId, kitty: Kitty },
		KittyTransferred { from: T::AccountId, to: T::AccountId, kitty_id: KittyId },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		InvalidKittyId,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(10_1000)]
		pub fn create(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let kitty_id = Self::get_next_id()?;
			ensure!(kitty_id != KittyId::max_value(), Error::<T>::StorageOverflow);
			let kitty = Kitty(Self::random_value(&who));
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
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(Kitties::<T>::contains_key(kitty_id_1), Error::<T>::InvalidKittyId);
			ensure!(Kitties::<T>::contains_key(kitty_id_2), Error::<T>::InvalidKittyId);
			let kitty_id = Self::get_next_id()?;
			ensure!(kitty_id != KittyId::max_value(), Error::<T>::StorageOverflow);

			let base_data = Self::random_value(&who);
			let mut new_kitty_data = [0u8; 16];

			let kitty1 = Kitties::<T>::get(kitty_id_1).ok_or(Error::<T>::InvalidKittyId)?;
			let kitty2 = Kitties::<T>::get(kitty_id_2).ok_or(Error::<T>::InvalidKittyId)?;

			for i in 0..kitty1.0.len() {
				new_kitty_data[i] = (kitty1.0[i] & base_data[i]) | (!kitty2.0[i] & !base_data[i]);
			}
			let kitty = Kitty(new_kitty_data);

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
	}
}
