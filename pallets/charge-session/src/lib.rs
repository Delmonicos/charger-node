#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Encode, Decode};

use pallet_timestamp as timestamp;

#[derive(Debug, PartialEq, Default, Encode, Decode)]
pub struct ChargeRequest<ChargerId, Moment> {
  charger_id: ChargerId,
  timestamp: Moment,
}

#[derive(Debug, PartialEq, Default, Encode, Decode)]
pub struct ChargingSession<ChargerId, Moment> {
  charger_id: ChargerId,
  started_at: Moment,
}

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
  use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use super::*;

  #[pallet::config]
  #[pallet::disable_frame_system_supertrait_check]
  pub trait Config: timestamp::Config {
    type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
  }

  #[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);


  #[pallet::storage]
  pub type UserRequests<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, ChargeRequest<T::AccountId, T::Moment>>;

  #[pallet::storage]
  pub type PendingSessions<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, ChargingSession<T::AccountId, T::Moment>>;

  #[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
  pub enum Event<T: Config> {
    // SessionStarted(User, Charger, Timestamp)
    SessionStarted(T::AccountId, T::AccountId, T::Moment),
    // SessionEnded(User, Charger, Timestamp)
    SessionEnded(T::AccountId, T::AccountId, T::Moment),
  }

  #[pallet::error]
  pub enum Error<T> {
    NoChargingRequest,
    NoChargingSession,
  }

  #[pallet::hooks]
  impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

  #[pallet::call]
  impl<T: Config> Pallet<T> {
    
    #[pallet::weight(1_000)]
    pub fn new_request(origin: OriginFor<T>, charger: T::AccountId) -> DispatchResultWithPostInfo {
      let sender = ensure_signed(origin)?;
      let now = <timestamp::Module<T>>::get();

      // Add the request to the storage with current timestamp
      UserRequests::<T>::insert(&sender, ChargeRequest{ charger_id: charger, timestamp: now });

      Ok(().into())
    }

    #[pallet::weight(1_000)]
    pub fn start_session(origin: OriginFor<T>, user: T::AccountId) -> DispatchResultWithPostInfo {
      let sender = ensure_signed(origin)?;
      let now = <timestamp::Module<T>>::get();

      // Validate that a request exists for this user & charger
      let request = UserRequests::<T>::get(&user);
      match request {
        None => return Err(Error::<T>::NoChargingRequest.into()),
        Some(request) if request.charger_id != sender => return Err(Error::<T>::NoChargingRequest.into()),
        _ => {},
      }
      // TODO: check timestamp for maximal period of time between new_request & start_session ?
      // TODO: check that this user does not have another active charging session

      // Remove the request from storage
      UserRequests::<T>::take(&user);

      // Add the pending charging session
      PendingSessions::<T>::insert(&user, ChargingSession{ charger_id: sender.clone(), started_at: now });

      // Emit an event
      Self::deposit_event(Event::SessionStarted(user, sender, now));

      Ok(().into())
    }

    #[pallet::weight(1_000)]
    pub fn end_session(origin: OriginFor<T>, user: T::AccountId) -> DispatchResultWithPostInfo {
      let sender = ensure_signed(origin)?;
      let now = <timestamp::Module<T>>::get();

      // Validate that a session exists for this user & charger
      let session = PendingSessions::<T>::get(&user);
      match session {
        None => return Err(Error::<T>::NoChargingSession.into()),
        Some(session) if session.charger_id != sender => return Err(Error::<T>::NoChargingSession.into()),
        _ => {},
      }
      
      // Remove the request from storage
      PendingSessions::<T>::take(&user);

      // TODO: offchain storage of charging time

      // Emit an event
      Self::deposit_event(Event::SessionEnded(user, sender, now));

      Ok(().into())
    }

  }

}
