#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod tests;

use serde::Serialize;
use codec::{Decode, Encode};
use core::convert::TryInto;
use frame_support::traits::Currency;
use pallet_timestamp as timestamp;
use sp_std::prelude::*;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PaymentHttpRequest<'a> {
    from_iban: &'a str,
    from_bic: &'a str,
    amount: u128,
}

#[derive(Debug, PartialEq, Default, Encode, Decode)]
pub struct PaymentConsent<Moment> {
    timestamp: Moment,
    iban: Vec<u8>,
    bic_code: Vec<u8>,
    signature: Vec<u8>,
}

#[derive(Debug, PartialEq, Default, Encode, Decode, Clone)]
pub struct Payment<Moment, Hash, AccountId> {
    timestamp: Moment,
    amount: u128,
    iban: Vec<u8>,
    bic_code: Vec<u8>,
    user_id: AccountId,
    session_id: Hash,
    charger_id: AccountId
}

pub mod crypto {
    use frame_system::offchain::AppCrypto;
    use sp_core::sr25519::Signature as SR25519Signature;
    use sp_runtime::{
        app_crypto::{app_crypto, sr25519},
        traits::Verify,
        KeyTypeId, MultiSignature, MultiSigner,
    };
    app_crypto!(sr25519, KeyTypeId(*b"paym"));

    pub struct PaymentValidatorId;

    impl frame_system::offchain::AppCrypto<MultiSigner, MultiSignature> for PaymentValidatorId {
        type RuntimeAppPublic = Public;
        type GenericSignature = sp_core::sr25519::Signature;
        type GenericPublic = sp_core::sr25519::Public;
    }

    impl AppCrypto<<SR25519Signature as Verify>::Signer, SR25519Signature> for PaymentValidatorId {
        type RuntimeAppPublic = Public;
        type GenericSignature = sp_core::sr25519::Signature;
        type GenericPublic = sp_core::sr25519::Public;
    }
}


pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::{
        offchain::{
            AppCrypto, CreateSignedTransaction, SendSignedTransaction, Signer, SigningTypes,
        },
        pallet_prelude::*
    };
    use sp_runtime::{
        traits::IdentifyAccount,
        RuntimeAppPublic,
        offchain as rt_offchain,
    };
    use pallet_registrar as registrar;
    use pallet_tariff_manager as tariff_manager;
    use pallet_charge_consent as consent;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub payment_validator_organization: T::AccountId,
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            <PaymentValidatorOrganization<T>>::put(&self.payment_validator_organization);
        }
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                payment_validator_organization: Default::default(),
            }
        }
    }
    
    #[pallet::config]
    #[pallet::disable_frame_system_supertrait_check]
    pub trait Config:
        frame_system::Config
        + CreateSignedTransaction<Call<Self>>
        + consent::Config
        + timestamp::Config
        + registrar::Config
        + tariff_manager::Config
    {
        type AuthorityId: AppCrypto<
            <Self as SigningTypes>::Public,
            <Self as SigningTypes>::Signature,
        >;
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn user_consents)]
    pub type PaymentConsents<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, PaymentConsent<T::Moment>>;

    #[pallet::storage]
    #[pallet::getter(fn pending_payments)]
    pub type PendingPayments<T: Config> = StorageValue<_, Vec<Payment<T::Moment, T::Hash, T::AccountId>>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn completed_payments)]
    pub type CompletedPayments<T: Config> = StorageMap<_, Blake2_128Concat, T::Hash, Payment<T::Moment, T::Hash, T::AccountId>>;

    #[pallet::storage]
    pub type AllowedUsers<T: Config> = StorageValue<_, Vec<(T::AccountId, Vec<u8>)>, ValueQuery>;

    #[pallet::storage]
    pub type PaymentValidatorOrganization<T: Config> = StorageValue<_, T::AccountId, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        // PaymentProcessed(User, Timestamp, u128, session_id)
        PaymentProcessed(T::AccountId, T::Moment, u128, T::Hash),
        // PaymentRequested(session_id)
        PaymentRequested(T::Hash),
        // UserConsentAdded(User, Timestamp, IBAN, bic)
        PaymentConsentAdded(T::AccountId, T::Moment, Vec<u8>, Vec<u8>, Vec<u8>),
        TariffRetrieved(Vec<u8>, u8),
    }

    #[pallet::error]
    pub enum Error<T> {
        NoConsentForPayment,
        //NoTariff,
        NotRegisteredPaymentValidator,
        AlreadyConfirmedPayment,
        NonExistentPayment,
    }
    
    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn offchain_worker(_block: T::BlockNumber) {
            // Offchain processing of pending payments
            Self::process_pending_payments();
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(1_000)]
        pub fn new_consent(
            origin: OriginFor<T>,
            iban: Vec<u8>,
            bic_code: Vec<u8>,
            signature: Vec<u8>, // hex encoded signature of the concatenation of iban and bic_code
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            let now = <timestamp::Module<T>>::get();

            // Add the user to the list of allowed users
            let mut vec = AllowedUsers::<T>::get();
            vec.push((sender.clone(), signature.clone()));
            AllowedUsers::<T>::put(vec);

            // Store the payment consent, linked to this user
            PaymentConsents::<T>::insert(
                &sender,
                PaymentConsent {
                    timestamp: now,
                    iban: iban.clone(),
                    bic_code: bic_code.clone(),
                    signature: signature.clone(),
                },
            );

            // Fire event
            Self::deposit_event(Event::PaymentConsentAdded(
                sender, now, iban, bic_code, signature,
            ));

            Ok(().into())
        }

        /*		#[pallet::weight(1_000)]
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
            let mut call = CallData::new( Selector::from_str("get_price") );
            //let input_data = call.to_bytes().to_vec();
            let input_data = Vec::from("");
            let _result = <contracts::Module<T>>::bare_call(sender.clone(), tariff_contract_adr, min_balance, u64::MAX, input_data);
            Self::deposit_event(Event::TariffRetrieved(Vec::from("fixed_price"), 123));

            Ok(().into())
        }*/

        #[pallet::weight(1_000)]
        pub fn process_payment(
            origin: OriginFor<T>,
            session_id: T::Hash,
            kwh: u128,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

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

            // Verify that this session_id has not already been confirmed
            if CompletedPayments::<T>::get(&session_id).is_some() {
                return Err(Error::<T>::AlreadyConfirmedPayment.into());
            }

            // Validate that a payment consent exists for this user
            let consent = PaymentConsents::<T>::get(&debtor);
            let (iban, bic_code) = match consent {
                None => return Err(Error::<T>::NoConsentForPayment.into()),
                Some(consent) => (consent.iban, consent.bic_code),
            };

            // Calculate the price.
            // Price is in cents

            let current_price = <tariff_manager::Module<T>>::get_current_price();
            let amount = kwh * current_price;

            // Add the payment request to the storage, for later processing by the offchain worker
            let mut pending_payments = PendingPayments::<T>::get();
            pending_payments.push(
                Payment {
                    timestamp: now,
                    amount: amount.clone(),
                    iban: iban.clone(),
                    bic_code: bic_code.clone(),
                    session_id: session_id.clone(),
                    charger_id: sender.clone(),
                    user_id: debtor,
                }
            );
            PendingPayments::<T>::put(pending_payments);
            
            // Emit an event
            Self::deposit_event(Event::PaymentRequested(session_id));

            Ok(().into())
        }

        #[pallet::weight(1_000)]
        pub fn complete_payment(
            origin: OriginFor<T>,
            session_id: T::Hash,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            ensure!(Self::is_payment_validator(&sender), Error::<T>::NotRegisteredPaymentValidator);

            // Verify that this session_id has not already been confirmed
            match CompletedPayments::<T>::get(&session_id) {
                Some(_) => return Err(Error::<T>::AlreadyConfirmedPayment.into()),
                None => {}
            };

            // Get the index of this payment in the list of pending payments
            let mut pending_payments = PendingPayments::<T>::get();
            let payment_index = pending_payments.iter().position(|p| p.session_id == session_id);
            
            let payment = match payment_index {
                None => {
                    // Cannot find the pending payment
                    return Err(Error::<T>::NonExistentPayment.into());
                },
                Some(index) => {
                    let payment = pending_payments.get(index).unwrap().clone();
                    // Remove the payment from pending payments
                    pending_payments.remove(index);
                    PendingPayments::<T>::put(pending_payments);
                    payment
                }
            };

            CompletedPayments::<T>::insert(session_id, payment.clone());
            Self::deposit_event(Event::PaymentProcessed(payment.user_id, payment.timestamp, payment.amount, session_id));

            Ok(().into())
        }

    }

    impl<T: Config> Pallet<T> {
        pub fn has_consent(who: &T::AccountId) -> bool {
            PaymentConsents::<T>::get(who).is_some()
        }

        pub fn nb_allowed() -> u32 {
            AllowedUsers::<T>::get().len().try_into().unwrap()
        }

        pub fn get_payment_consents() -> Vec<(Vec<u8>, Vec<u8>)> {
            let v = AllowedUsers::<T>::get();
            v.into_iter()
                .map(|key| {
                    let consent = match PaymentConsents::<T>::get(key.0.clone()) {
                        Some(cs) => cs.iban,
                        None => "".as_bytes().to_vec(),
                    };
                    (key.encode(), consent)
                })
                .collect()
        }

        pub fn is_payment_validator(who: &T::AccountId) -> bool {
            return <pallet_registrar::Module<T>>::members_of(<PaymentValidatorOrganization<T>>::get())
                .contains(who);
        }

        fn process_pending_payments() {
            // Get the first payment validator account
            let account = <<T as Config>::AuthorityId as AppCrypto<
                <T as SigningTypes>::Public,
                <T as SigningTypes>::Signature,
            >>::RuntimeAppPublic::all()
            .into_iter()
            .map(|key| {
                let generic_public = <<T as Config>::AuthorityId as AppCrypto<
                    <T as SigningTypes>::Public,
                    <T as SigningTypes>::Signature,
                >>::GenericPublic::from(key);
                let public: <T as SigningTypes>::Public = generic_public.into();
                let signer = Signer::<T, <T as Config>::AuthorityId>::all_accounts()
                    .with_filter(sp_std::vec!(public.clone()));
                (public.clone().into_account(), signer)
            }).next();

            match account {
                None => {
                    debug::native::debug!("No payment validator account configured on this node");
                },
                Some((account_id, signer)) => {
                    debug::native::debug!("Use payment validator account {}", account_id);
                    // Process all pending payment
                    for payment in Self::pending_payments() {
                        debug::native::debug!("Process payment for session_id {}", &payment.session_id);

                        let payment_consent = PaymentConsents::<T>::get(&payment.user_id).unwrap();
                        let session_id = payment.session_id;

                        match Self::request_payment(
                            payment_consent.iban,
                            payment_consent.bic_code,
                            payment.amount,
                        ) {
                            Err(_) => {
                                debug::native::error!("An error occured during HTTP call for payment session {}", &&session_id);
                                continue
                            },
                            _ => {
                                debug::native::info!("HTTP call for payment session {} processed", &&session_id);
                            }
                        }

                        let completion_status = match signer.send_signed_transaction(|_| Call::complete_payment(session_id)).as_slice() {
                            [(_, result)] => *result,
                            _ => Err(()),
                        };

                        if completion_status.is_err() {
                            debug::native::error!("Error occured when sending signed transaction for session_id {}", &payment.session_id);
                        }
                        else {
                            debug::native::info!("Payment for session_id {} completed", &payment.session_id);
                        }
                    }
                }
            }
        }

        fn request_payment(iban: Vec<u8>, bic: Vec<u8>, amount: u128) -> Result<(), &'static str> {

            let http_request = PaymentHttpRequest {
                from_iban: sp_std::str::from_utf8(&iban).unwrap(),
                from_bic: sp_std::str::from_utf8(&bic).unwrap(),
                amount: amount,
            };
            let serialized = serde_json::to_string(&http_request).unwrap();
            debug::native::debug!("payment request: {}", &serialized);

            let body = serialized.as_bytes().to_vec();
            let timeout = sp_io::offchain::timestamp().add(rt_offchain::Duration::from_millis(10000));
            let request = rt_offchain::http::Request::post(
                "https://app-9b140c2a-277a-4cbb-96a6-6af5d21b0fe9.cleverapps.io/payments",
                vec![body]
            );

            let pending = request
                .add_header("Content-Type", "application/json")
                .deadline(timeout) 
                .send()
                .map_err(|_| "error on HTTP call")?;

            let response = pending
                .try_wait(timeout)
                .map_err(|_| "error on HTTP call")?
                .map_err(|_| "error on HTTP call")?;
            
            if response.code != 200 {
                debug::native::error!("Unexpected http request status code: {}", response.code);
                return Err("error on HTTP call")
            }
            Ok(())
        }
    }
}
