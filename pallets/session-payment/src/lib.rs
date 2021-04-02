#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod tests;

use codec::{Decode, Encode};

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
pub struct PaymentExecution<Moment> {
    timestamp: Moment,
    amount: Currency,
    iban: SubsStr,
    bic_code: SubsStr,
}

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use pallet_user_consent as consent;
	use pallet_registrar as registrar;

    #[pallet::config]
    #[pallet::disable_frame_system_supertrait_check]
    pub trait Config: frame_system::Config
	+ consent::Config
	+ timestamp::Config
	+ registrar::Config
	{
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn user_consents)]
    pub type UserConsents<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, UserConsent<T::Moment>>;

    #[pallet::storage]
    #[pallet::getter(fn user_payments)]
    pub type UserPayments<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, PaymentExecution<T::Moment>>;

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
        PaymentError,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(1_000)]
        pub fn new_consent(
            origin: OriginFor<T>,
            iban: SubsStr,
            bic_code: SubsStr,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            let now = <timestamp::Module<T>>::get();

            // Add the request to the storage with current timestamp

            UserConsents::<T>::insert(
                &sender,
                UserConsent {
                    timestamp: now,
                    iban: iban.clone(),
                    bic_code: bic_code.clone(),
                },
            );

            // Fire event
            Self::deposit_event(Event::UserConsentAdded(sender, now, iban, bic_code));

            Ok(().into())
        }

        #[pallet::weight(1_000)]
        pub fn process_payment(
            origin: OriginFor<T>,
            session_id: T::Hash,
            kwh: u128,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

			// Verify that the sender is a charger
			// ensure!(<registrar::Module<T>>::members_of(<ChargerOrganization<T>>::get()).contains(&sender), Error::<T>::NoConsentForPayment);

            let now = <timestamp::Module<T>>::get();

            // Verify that there is a session_id corresponding
            let debtor = match <consent::Module<T>>::get_consent_from_session_id(session_id) {
                None => return Err(Error::<T>::NoConsentForPayment.into()),
                Some(session) => {
                    if session.charger_id != sender {
                        return Err(Error::<T>::NoConsentForPayment.into());
                    } else {
                        session.user_id
                    }
                }
            };

            // Validate that a request exists for this user & charger
            let consent = UserConsents::<T>::get(&debtor);
            let (iban, bic_code) = match consent {
                None => return Err(Error::<T>::NoConsentForPayment.into()),
                Some(consent) => (consent.iban, consent.bic_code),
            };

            // Calculate the price.
			// For instance, we consider a fixed price of 0,15 €/kwh
			// Price in in cents

			let amount = kwh * 15;

			// TODO: Execute payment


            // Add the request to the storage with current timestamp
            UserPayments::<T>::insert(
                &debtor,
                PaymentExecution {
                    timestamp: now,
                    amount: amount.clone(),
                    iban: iban.clone(),
                    bic_code: bic_code.clone(),
                },
            );
            // Emit an event
            Self::deposit_event(Event::PaymentProcessed(debtor, now, amount));

            Ok(().into())
        }

        // TODO: this can be a public function (not in #[pallet::call])
        #[pallet::weight(1_000)]
        pub fn is_allowed_to_pay(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            // Validate that a request exists for this user & charger
            match UserConsents::<T>::get(&sender) {
                None => Err(Error::<T>::NoConsentForPayment.into()),
                Some(_consent) => Ok(().into()),
            }
        }
    }
    
    impl<T: Config> Pallet<T> {
        pub fn has_consent(who: &T::AccountId) -> bool {
            UserConsents::<T>::get(who).is_some()
        }
    }
}
