#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod tests;

use codec::{Decode, Encode};

#[derive(Debug, PartialEq, Default, Encode, Decode)]
pub struct ChargeRequest<UserId, Moment, Hash> {
    user_id: UserId,
    created_at: Moment,
    session_id: Hash,
}

#[derive(Debug, PartialEq, Default, Encode, Decode)]
pub struct ChargingSession<UserId, Moment, Hash> {
    user_id: UserId,
    started_at: Moment,
    session_id: Hash,
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

#[frame_support::pallet]
pub mod pallet {
    use super::{ChargeRequest, ChargingSession};
    use charger_service::runtime::offchain::{api as charger_api, ChargeStatus};
    use frame_support::pallet_prelude::*;
    use frame_system::{
        offchain::{
            AppCrypto, CreateSignedTransaction, SendSignedTransaction, Signer, SigningTypes,
        },
        pallet_prelude::*,
    };
    use pallet_registrar as registrar;
    use pallet_did as did;
    use pallet_timestamp as timestamp;
    use pallet_charge_consent as consent;
    use sp_runtime::{
        traits::{Hash, IdentifyAccount},
        RuntimeAppPublic,
    };
    use sp_std::vec::Vec;
    use pallet_did::did::Did;


	#[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub charger_organization: T::AccountId,
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            <ChargerOrganization<T>>::put(&self.charger_organization);
        }
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                charger_organization: Default::default(),
            }
        }
    }

    #[pallet::config]
    #[pallet::disable_frame_system_supertrait_check]
    pub trait Config:
        frame_system::Config
        + CreateSignedTransaction<Call<Self>>
        + registrar::Config
        + did::Config
        + timestamp::Config
        + consent::Config
        + pallet_session_payment::Config
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
    #[pallet::getter(fn user_requests)]
    pub type UserRequests<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        ChargeRequest<T::AccountId, T::Moment, T::Hash>,
    >;

    #[pallet::storage]
    #[pallet::getter(fn active_sessions)]
    pub type ActiveSessions<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        ChargingSession<T::AccountId, T::Moment, T::Hash>,
    >;

    #[pallet::storage]
    pub type ChargerOrganization<T: Config> = StorageValue<_, T::AccountId, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// SessionRequested(User, Charger, Timestamp, SessionId)
        SessionRequested(T::AccountId, T::AccountId, T::Moment, T::Hash),
        /// SessionStarted(User, Charger, Timestamp, SessionId)
        SessionStarted(T::AccountId, T::AccountId, T::Moment, T::Hash),
        /// SessionEnded(User, Charger, Timestamp, SessionId, kwh)
        SessionEnded(T::AccountId, T::AccountId, T::Moment, T::Hash, u64),
        // NewChargerAdded(AddedBy, ChargerId, Location)
        NewChargerAdded(T::AccountId, T::AccountId, Vec<u8>),
    }

    #[pallet::error]
    pub enum Error<T> {
        NotAnAdmin,
        NoLocation,
        NotRegisteredCharger,
        NoChargingRequest,
        NoChargingSession,
        ChargerIsBusy,
        NoPaymentConsent,
        AlreadyRegisteredCharger,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn offchain_worker(_block: T::BlockNumber) {
            // Offchain processing of charge requests & active charge sessions
            Self::process_charge_sessions();
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T>
	{
        #[pallet::weight(1_000)]
        pub fn new_request(
            origin: OriginFor<T>,
            charger: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin.clone())?;
            ensure!(Self::is_charger(&charger), Error::<T>::NotRegisteredCharger);

            let now = <timestamp::Module<T>>::get();

            // Check that sender consent exists in pallet_session_payment
            if <pallet_session_payment::Module<T>>::has_consent(&sender) == false {
                debug::native::warn!("No consent for user {}", &sender,);
                return Err(Error::<T>::NoPaymentConsent.into());
            }

            // Check that this charger does not have another pending request
            // TODO: expiration period for request?
            match UserRequests::<T>::get(&charger) {
                Some(request) => {
                    debug::native::warn!(
                        "Charger {} has pending request {:?}: cannot store a new request",
                        &charger,
                        &request
                    );
                    return Err(Error::<T>::ChargerIsBusy.into());
                }
                _ => {}
            }

            // Check that this charger does not have an active charging session
            match ActiveSessions::<T>::get(&charger) {
                Some(session) => {
                    debug::native::warn!(
                        "Charger {} has already active session {:?}: cannot store a new request",
                        &charger,
                        &session
                    );
                    return Err(Error::<T>::ChargerIsBusy.into());
                }
                _ => {}
            }

            // Generate a new session_id
            let session_id = Self::generate_charge_id(&sender, &charger);

            // Store the charge consent
            <consent::Module<T>>::new_consent_for_user(
                origin,
                charger.clone(),
                session_id.clone(),
            )?;

            // Add the request to the storage with current timestamp
            UserRequests::<T>::insert(
                &charger,
                ChargeRequest {
                    user_id: sender.clone(),
                    created_at: now,
                    session_id,
                },
            );

            Self::deposit_event(Event::SessionRequested(sender, charger, now, session_id));

            Ok(().into())
        }

        #[pallet::weight(1_000)]
        pub fn start_session(
            origin: OriginFor<T>,
            user: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            ensure!(Self::is_charger(&sender), Error::<T>::NotRegisteredCharger);

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
            let request = UserRequests::<T>::take(&sender).expect("cannot be None");

            // Add the pending charging session
            ActiveSessions::<T>::insert(
                &sender,
                ChargingSession {
                    user_id: user.clone(),
                    started_at: now,
                    session_id: request.session_id,
                },
            );

            // Emit an event
            Self::deposit_event(Event::SessionStarted(user, sender, now, request.session_id));

            Ok(().into())
        }

        #[pallet::weight(1_000)]
        pub fn end_session(
            origin: OriginFor<T>,
            user: T::AccountId,
            kwh: u64,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin.clone())?;
            ensure!(Self::is_charger(&sender), Error::<T>::NotRegisteredCharger);

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
            let session = ActiveSessions::<T>::take(&sender).expect("Cannot be None");

            // Execute the payment
			// TODO Uncomment the following code to execute the payment
            match <pallet_session_payment::Module<T>>::process_payment(
                origin,
                session.session_id,
                kwh.into(),
            ) {
                Err(error) => {
                    // The error is just logged here, because we want to end the session even if payment has failed
                    // pallet_session_payment deposits an error event which is handled manually in this case
                    debug::native::error!(
                        "An error occured in pallet_session_payment::process_payment for session id {}: {:?}",
                        &session.session_id,
                        error
                    );
                }
                _ => {}
            }

            // Emit an event
            Self::deposit_event(Event::SessionEnded(
                user,
                sender,
                now,
                session.session_id,
                kwh,
            ));

            Ok(().into())
        }

        #[pallet::weight(1_000)]
        pub fn add_new_charger(
            origin: OriginFor<T>,
            charger_id: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin.clone())?;
            // Check that signer is admin (= owner of chargers organizaton)
            ensure!(<pallet_did::Module<T>>::is_owner(&<ChargerOrganization<T>>::get(), &sender).is_ok(), Error::<T>::NotAnAdmin);
            // Check that this charger is not already registered
            ensure!(Self::is_charger(&charger_id) == false, Error::<T>::AlreadyRegisteredCharger);
             match <pallet_did::Module<T>>::attribute_and_id(&charger_id, b"location") {
                 // Check that charger has a location attribute
                None =>  return Err(Error::<T>::NoLocation.into()),
                Some(location) => {
                    // Add charger to organization
                    <pallet_registrar::Module<T>>::add_to_organization(origin.clone(), charger_id.clone())?;
                    
                    // Emit an event
                    Self::deposit_event(Event::NewChargerAdded(sender, charger_id, location.0.value));
                    Ok(().into())
                }
             }
        }
    }

    impl<T: Config> Pallet<T> {
        fn process_charge_sessions() {
            // Get the list of charger accounts
            let accounts = <T::AuthorityId as AppCrypto<
                <T as SigningTypes>::Public,
                <T as SigningTypes>::Signature,
            >>::RuntimeAppPublic::all()
            .into_iter()
            .map(|key| {
                let generic_public = <T::AuthorityId as AppCrypto<
                    <T as SigningTypes>::Public,
                    <T as SigningTypes>::Signature,
                >>::GenericPublic::from(key);
                let public: <T as SigningTypes>::Public = generic_public.into();
                let signer = Signer::<T, T::AuthorityId>::all_accounts()
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
                                debug::native::error!(
                                    "Charge session is active in-chain, but not found off-chain"
                                );
                            }
                            ChargeStatus::Active => {
                                debug::native::debug!("Charge session is still active, waiting...");
                            }
                            ChargeStatus::Ended { kwh } => {
                                debug::native::info!(
                                    "Charge session is ended for user {}, consumed: {} kwh",
                                    &session.user_id,
                                    &kwh
                                );
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

        pub fn is_charger(who: &T::AccountId) -> bool {
            return <pallet_registrar::Module<T>>::members_of(<ChargerOrganization<T>>::get())
                .contains(who);
        }

        fn send_signed_transaction(
            signer: &Signer<T, T::AuthorityId, frame_system::offchain::ForAll>,
            call: Call<T>,
        ) -> Result<(), ()> {
            match signer.send_signed_transaction(|_| call.clone()).as_slice() {
                [(_, result)] => *result,
                _ => Err(()),
            }
        }

        /// Generate a new charge session id
        /// ID is a hash of the concatenation of
        ///     - current block number
        ///     - AccountId of the user (charge requester)
        ///     - AccountId of the charger
        /// since there is only one possible session for a charger and user at any given time, this identifier is guaranteed to be unique
        fn generate_charge_id(user: &T::AccountId, charger: &T::AccountId) -> T::Hash {
            let block_number = <frame_system::Pallet<T>>::block_number();
            let mut key = T::AccountId::encode(user);
            key.append(&mut T::AccountId::encode(charger));
            key.append(&mut T::BlockNumber::encode(&block_number));
            let hash = T::Hashing::hash(&key);
            return hash;
        }
    }
}

pub use pallet::*;
