#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};

use pallet_timestamp as timestamp;

#[cfg(test)]
mod tests;

use charger_service::runtime::offchain::{api as charger_api, ChargeStatus};

#[derive(Debug, PartialEq, Default, Encode, Decode)]
pub struct ChargeRequest<UserId, Moment> {
    user_id: UserId,
    created_at: Moment,
}

#[derive(Debug, PartialEq, Default, Encode, Decode)]
pub struct ChargingSession<UserId, Moment> {
    user_id: UserId,
    started_at: Moment,
}

pub mod crypto {
    use frame_system::offchain::AppCrypto;
    use sp_core::sr25519::Signature as SR25519Signature;
    use sp_runtime::{
        app_crypto::{app_crypto, sr25519},
        traits::Verify,
        KeyTypeId, MultiSignature, MultiSigner,
    };
    app_crypto!(sr25519, KeyTypeId(*b"chrg"));

    pub struct ChargerId;

    impl frame_system::offchain::AppCrypto<MultiSigner, MultiSignature> for ChargerId {
        type RuntimeAppPublic = Public;
        type GenericSignature = sp_core::sr25519::Signature;
        type GenericPublic = sp_core::sr25519::Public;
    }

    impl AppCrypto<<SR25519Signature as Verify>::Signer, SR25519Signature> for ChargerId {
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
        offchain::{AppCrypto, CreateSignedTransaction, SendSignedTransaction, Signer},
        pallet_prelude::*,
    };
    use sp_runtime::{traits::IdentifyAccount, RuntimeAppPublic};

    #[pallet::config]
    #[pallet::disable_frame_system_supertrait_check]
    pub trait Config:
        frame_system::Config + CreateSignedTransaction<Call<Self>> + timestamp::Config
    {
        type AutorityId: AppCrypto<Self::Public, Self::Signature>;
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn user_requests)]
    pub type UserRequests<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, ChargeRequest<T::AccountId, T::Moment>>;

    #[pallet::storage]
    #[pallet::getter(fn active_sessions)]
    pub type ActiveSessions<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, ChargingSession<T::AccountId, T::Moment>>;

    #[pallet::event]
    //#[pallet::metadata(T::AccountId = "AccountId", T::Moment = "Timestamp")]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// SessionRequested(User, Charger, Timestamp)
        SessionRequested(T::AccountId, T::AccountId, T::Moment),
        /// SessionStarted(User, Charger, Timestamp)
        SessionStarted(T::AccountId, T::AccountId, T::Moment),
        /// SessionEnded(User, Charger, Timestamp, kwh)
        SessionEnded(T::AccountId, T::AccountId, T::Moment, u64),
    }

    #[pallet::error]
    pub enum Error<T> {
        NoChargingRequest,
        NoChargingSession,
        ChargerIsBusy,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn offchain_worker(_block: T::BlockNumber) {
            // Offchain processing of charge requests & active charge sessions
            Self::process_charge_sessions();
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(1_000)]
        pub fn new_request(
            origin: OriginFor<T>,
            charger: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            let now = <timestamp::Module<T>>::get();

            // Check that this charger does not have another pending request
            // TODO: expiration period for request?
            match UserRequests::<T>::get(&charger) {
                Some(request) => {
                    debug::native::warn!("Charger {} has pending request {:?}: cannot store a new request", &charger, &request);
                    return Err(Error::<T>::ChargerIsBusy.into());
                },
                _ => {}
            }

            // Check that this charger does not have an active charging session
            match ActiveSessions::<T>::get(&charger) {
                Some(session) => {
                    debug::native::warn!("Charger {} has already active session {:?}: cannot store a new request", &charger, &session);
                    return Err(Error::<T>::ChargerIsBusy.into());
                },
                _ => {}
            }

            // Add the request to the storage with current timestamp
            UserRequests::<T>::insert(
                &charger,
                ChargeRequest {
                    user_id: sender.clone(),
                    created_at: now,
                },
            );

            Self::deposit_event(Event::SessionRequested(sender, charger, now));

            Ok(().into())
        }

        #[pallet::weight(1_000)]
        pub fn start_session(
            origin: OriginFor<T>,
            user: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            let now = <timestamp::Module<T>>::get();

            // Validate that a request exists for this user & charger
            match UserRequests::<T>::get(&sender) {
                None => return Err(Error::<T>::NoChargingRequest.into()),
                Some(request) if request.user_id != user => {
                    return Err(Error::<T>::NoChargingRequest.into())
                }
                _ => {}
            }
            // TODO: check timestamp for maximal period of time between new_request & start_session ?

            // Remove the request from storage
            UserRequests::<T>::take(&sender);

            // Add the pending charging session
            ActiveSessions::<T>::insert(
                &sender,
                ChargingSession {
                    user_id: user.clone(),
                    started_at: now,
                },
            );

            // Emit an event
            Self::deposit_event(Event::SessionStarted(user, sender, now));

            Ok(().into())
        }

        #[pallet::weight(1_000)]
        pub fn end_session(origin: OriginFor<T>, user: T::AccountId, kwh: u64) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            let now = <timestamp::Module<T>>::get();

            // Validate that a session exists for this user & charger
            match ActiveSessions::<T>::get(&sender) {
                None => return Err(Error::<T>::NoChargingSession.into()),
                Some(session) if session.user_id != user => {
                    return Err(Error::<T>::NoChargingSession.into())
                }
                _ => {}
            }

            // Remove the request from storage
            ActiveSessions::<T>::take(&sender);

            // Emit an event
            Self::deposit_event(Event::SessionEnded(user, sender, now, kwh));

            Ok(().into())
        }
    }

    impl<T: Config> Pallet<T> {
        fn process_charge_sessions() {
            // Get the list of charger accounts
            let accounts =
                <T::AutorityId as AppCrypto<T::Public, T::Signature>>::RuntimeAppPublic::all()
                    .into_iter()
                    .map(|key| {
                        let generic_public = <T::AutorityId as AppCrypto<
                            T::Public,
                            T::Signature,
                        >>::GenericPublic::from(key);
                        let public: T::Public = generic_public.into();
                        let signer = Signer::<T, T::AutorityId>::all_accounts()
                            .with_filter(sp_std::vec!(public.clone()));
                        (public.clone().into_account(), signer)
                    });

            // For each charger account registered in the keystore:
            for (account_id, signer) in accounts {
                debug::native::debug!("Use charger account {}", account_id);

                // 1) Check if pending user request exists for this charger
                match Self::user_requests(&account_id) {
                    Some(request) => {
                        debug::native::debug!(
                            "User {} requests a new charge session",
                            &request.user_id
                        );
                        if charger_api::start_charge() {
                            debug::native::info!(
                                "Charge session started for user {}",
                                &request.user_id
                            );
                            if Self::send_signed_transaction(
                                &signer,
                                Call::start_session(request.user_id.clone()),
                            )
                            .is_err()
                            {
                                debug::native::error!(
                                    "Error occured while sending start_session transaction"
                                );
                            }
                        }
                    }
                    _ => {}
                }

                // 2) Check if there is an active charge session for this charger
                match Self::active_sessions(&account_id) {
                    Some(session) => {
                        // We have an active session, check the current status
                        match charger_api::get_current_charge_status() {
                            ChargeStatus::NoCharge => {
                                debug::native::error!("Charge session is active in-chain, but not found off-chain");
                            },
                            ChargeStatus::Active => {
                                debug::native::debug!("Charge session is still active, waiting...");
                            },
                            ChargeStatus::Ended{ kwh } => {
                                debug::native::info!("Charge session is ended for user {}, consumed: {} kwh", &session.user_id, &kwh);
                                if Self::send_signed_transaction(
                                    &signer,
                                    Call::end_session(session.user_id.clone(), kwh),
                                )
                                .is_err()
                                {
                                    debug::native::error!(
                                        "Error occured while sending end_session transaction"
                                    );
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        fn send_signed_transaction(
            signer: &Signer<T, T::AutorityId, frame_system::offchain::ForAll>,
            call: Call<T>,
        ) -> Result<(), ()> {
            match signer.send_signed_transaction(|_| call.clone()).as_slice() {
                [(_, result)] => *result,
                _ => Err(()),
            }
        }
    }
}
