#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod tests;

use codec::{Decode, Encode};

#[derive(Debug, PartialEq, Default, Encode, Decode)]
pub struct SessionConsent<UserId, ChargerId> {
    pub user_id: UserId,
    pub charger_id: ChargerId,
}

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use super::SessionConsent;

    #[pallet::config]
    pub trait Config: frame_system::Config {
      type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn user_consent)]
    pub type UserConsent<T: Config> = StorageMap<_, Blake2_128Concat, T::Hash, SessionConsent<T::AccountId, T::AccountId>>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// NewConsent(User, Charger, SessionId)
        ConsentStored(T::AccountId, T::AccountId, T::Hash),
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(1_000)]
        pub fn new_consent_for_user(
            origin: OriginFor<T>,
            charger: T::AccountId,
            session_id: T::Hash
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            // TODO: check that charger is a charger (using membership pallet!)

            // Add the request to the storage with current timestamp
            UserConsent::<T>::insert(
                &session_id,
                SessionConsent {
                    user_id: sender.clone(),
                    charger_id: charger.clone(),
                },
            );

            Self::deposit_event(Event::ConsentStored(sender, charger, session_id));

            Ok(().into())
        }
    }

	impl<T: Config> Pallet<T> {
		pub fn get_consent_from_session_id(session_id: T::Hash) -> Option<SessionConsent<T::AccountId, T::AccountId>> {
			UserConsent::<T>::get(&session_id)
		}
	}
}

pub use pallet::*;
