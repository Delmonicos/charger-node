#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Encode, Decode};

use pallet_timestamp as timestamp;
use sp_std::prelude::*;


type Currency = u128;
type SubsStr = Vec<u8>;

#[derive(Debug, PartialEq, Default, Encode, Decode)]
pub struct UserConsent<Moment> {
	timestamp: Moment,
	iban: SubsStr,
	bic_code: SubsStr,
}

#[derive(Debug, PartialEq, Default, Encode, Decode)]
pub struct PaymentRequest<AccountId, Moment> {
	account_id: AccountId,
	timestamp: Moment,
	amount: Currency
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
	pub type UserConsents<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, UserConsent<T::Moment>>;


	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		// PaymentProcessed(User, Timestamp, currency)
		PaymentProcessed(T::AccountId, T::Moment, Currency),
		// UserConsentAdded(User, Timestamp, IBAN, bic)
		UserConsentAdded(T::AccountId, T::Moment, SubsStr, SubsStr),
	}

	#[pallet::error]
	pub enum Error<T> {
		NoConsentForPayment,
		UserConsentRefused,
		PaymentError
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {

		#[pallet::weight(1_000)]
		pub fn new_consent(origin: OriginFor<T>, iban: SubsStr, bic_code: SubsStr) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;
			let now = <timestamp::Module<T>>::get();

			// Add the request to the storage with current timestamp
			UserConsents::<T>::insert(&sender, UserConsent{timestamp: now, iban: iban.clone(), bic_code: bic_code.clone() });

			// Fire event
			Self::deposit_event(Event::UserConsentAdded(sender, now, iban, bic_code));

			Ok(().into())
		}

		#[pallet::weight(1_000)]
		pub fn process_payment(origin: OriginFor<T>, amount: Currency) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;
			let now = <timestamp::Module<T>>::get();

			// Validate that a request exists for this user & charger
			let consent = UserConsents::<T>::get(&sender);
			let (_iban, _bic_code) = match consent {
				None => return Err(Error::<T>::NoConsentForPayment.into()),
				Some(consent) => (consent.iban, consent.bic_code),
			//	_ => {},
			};

			// TODO: Execute payment

			// Emit an event
			Self::deposit_event(Event::PaymentProcessed(sender, now, amount));

			Ok(().into())
		}


	}

}
