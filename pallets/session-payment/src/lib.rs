#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod tests;

use codec::{Decode, Encode};
use core::convert::TryInto;
use pallet_timestamp as timestamp;
use sp_std::prelude::*;
use frame_support::traits::Currency;


#[derive(Debug, PartialEq, Default, Encode, Decode)]
pub struct UserConsent<Moment> {
    timestamp: Moment,
    iban: Vec<u8>,
    bic_code: Vec<u8>,
    signature: Vec<u8>,
}

#[derive(Debug, PartialEq, Default, Encode, Decode)]
pub struct PaymentExecution<Moment> {
    timestamp: Moment,
    amount: u128,
    iban: Vec<u8>,
    bic_code: Vec<u8>,
}

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use pallet_contracts as contracts;
    use pallet_registrar as registrar;
    use pallet_tariff_manager as tariff_manager;
    use pallet_user_consent as consent;
	use sp_core::crypto::UncheckedFrom;



	#[pallet::config]
    #[pallet::disable_frame_system_supertrait_check]
    pub trait Config:
        frame_system::Config
        + consent::Config
        + timestamp::Config
        + registrar::Config
        + tariff_manager::Config
        + contracts::Config
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

    #[pallet::storage]
    pub type AllowedUsers<T: Config> = StorageValue<_, Vec<(T::AccountId, Vec<u8>)>, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        // PaymentProcessed(User, Timestamp, u128)
        PaymentProcessed(T::AccountId, T::Moment, u128),
        // UserConsentAdded(User, Timestamp, IBAN, bic)
        UserConsentAdded(T::AccountId, T::Moment, Vec<u8>, Vec<u8>, Vec<u8>),
		TariffRetrieved(Vec<u8>, u8),
    }

    #[pallet::error]
    pub enum Error<T> {
        NoConsentForPayment,
        UserConsentRefused,
        NoTariff,
        PaymentError,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T>
    where
        T::AccountId: UncheckedFrom<T::Hash>,
        T::AccountId: AsRef<[u8]>,
    {
        #[pallet::weight(1_000)]
        pub fn new_consent(
            origin: OriginFor<T>,
            iban: Vec<u8>,
            bic_code: Vec<u8>,
            signature: Vec<u8>, // hex encoded signature of the concatenation of iban and bic_code
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            let now = <timestamp::Module<T>>::get();

            // Add the request to the storage with current timestamp

            let mut vec = AllowedUsers::<T>::get();
            vec.push((sender.clone(), signature.clone()));
            AllowedUsers::<T>::put(vec);

            UserConsents::<T>::insert(
                &sender,
                UserConsent {
                    timestamp: now,
                    iban: iban.clone(),
                    bic_code: bic_code.clone(),
                    signature: signature.clone(),
                },
            );

            // Fire event
            Self::deposit_event(Event::UserConsentAdded(
                sender, now, iban, bic_code, signature,
            ));

            Ok(().into())
        }


		#[pallet::weight(1_000)]
		pub fn process_tariff(
			origin: OriginFor<T>,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			let tariff_contract_adr =
				match <tariff_manager::Module<T>>::get_tariff(Vec::from("fixed_price")) {
					None => return Err(Error::<T>::NoTariff.into()),
					Some(contract_adr) => contract_adr,
				};

			let min_balance = <T as pallet_contracts::Config>::Currency::minimum_balance();
			// let mut call = CallData::new( Selector::from_str("get_price") );
			//let input_data = call.to_bytes().to_vec();
			let input_data = Vec::from("");
			let _result = <contracts::Module<T>>::bare_call(sender.clone(), tariff_contract_adr, min_balance, u64::MAX, input_data);
			Self::deposit_event(Event::TariffRetrieved(Vec::from("fixed_price"), 123));

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

			let current_price = <tariff_manager::Module<T>>::get_current_price();
            let amount = kwh * current_price;

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

        pub fn nb_allowed() -> u32 {
            AllowedUsers::<T>::get().len().try_into().unwrap()
        }

        pub fn get_payment_consents() -> Vec<(Vec<u8>, Vec<u8>)> {
            let v = AllowedUsers::<T>::get();
            v.into_iter()
                .map(|key| {
                    let consent = match UserConsents::<T>::get(key.0.clone()) {
                        Some(cs) => cs.iban,
                        None => "".as_bytes().to_vec(),
                    };
                    (key.encode(), consent)
                })
                .collect()
        }

    }
}
