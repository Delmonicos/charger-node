#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod tests;

use pallet_timestamp as timestamp;
use sp_std::prelude::*;


pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;


	#[pallet::config]
    pub trait Config:
        frame_system::Config + timestamp::Config
    {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn available_tariffs)]
    pub type AvailableTariffs<T: Config> =
        StorageMap<_, Blake2_128Concat, Vec<u8>, T::AccountId>;

	#[pallet::storage]
	pub type CurrentPrice<T: Config> = StorageValue<_, u128, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        TariffAdded(Vec<u8>, T::AccountId, T::Moment),
		PriceModified(u128),
    }

    #[pallet::error]
    pub enum Error<T> {
        NoTariff,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(1_000)]
        pub fn new_tariff(
            origin: OriginFor<T>,
			label: Vec<u8>,
            tariff: T::AccountId
        ) -> DispatchResultWithPostInfo {
            let _sender = ensure_signed(origin)?;
			let now = <timestamp::Module<T>>::get();

			AvailableTariffs::<T>::insert(
                &label,
                tariff.clone()
            );

            // Fire event
            Self::deposit_event(Event::TariffAdded(label, tariff, now));

            Ok(().into())
        }

		#[pallet::weight(1_000)]
		pub fn set_current_price(
			origin: OriginFor<T>,
			new_price: u128,
		) -> DispatchResultWithPostInfo {
			let _sender = ensure_signed(origin)?;

			CurrentPrice::<T>::put(new_price);

			// Fire event
			Self::deposit_event(Event::PriceModified(new_price));

			Ok(().into())
		}

    }

	impl<T: Config> Pallet<T> {
		pub fn get_tariff(
			label: Vec<u8>,
		) -> Option<T::AccountId> {
			AvailableTariffs::<T>::get(&label)
		}

		pub fn get_current_price() -> u128 {
			CurrentPrice::<T>::get()
		}
	}
}
