use crate as pallet_session_payment;

use frame_support::{assert_err, assert_ok, traits::GenesisBuild};
use sp_core::{sr25519::Signature, H256};
use sp_io::TestExternalities;
use sp_runtime::{
    testing::{Header, TestXt},
    traits::{BlakeTwo256, Extrinsic as ExtrinsicT, Hash, IdentifyAccount, IdentityLookup, Verify},
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

pub fn new_test_ext() -> TestExternalities {
    let mut storage = frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();
    pallet_session_payment::GenesisConfig::<Test> {
        payment_validator_organization: Public::from_raw(hex!(
            "108fa7489d496834f3cbbc690798dbda53cf8edc6781672f706a031afcfb811f"
        )),
    }
    .assimilate_storage(&mut storage)
    .unwrap();
    storage.into()
}

frame_support::construct_runtime!(
  pub enum Test where
    Block = Block,
    NodeBlock = Block,
    UncheckedExtrinsic = UncheckedExtrinsic,
  {
    System: frame_system::{Module, Call, Config, Storage, Event<T>},
    Timestamp: pallet_timestamp::{Module, Call, Storage, Inherent},
    SessionPayment: pallet_session_payment::{Module, Call, Storage, Event<T>},
    ChargeConsent: pallet_charge_consent::{Module, Call, Storage, Event<T>},
    DID: pallet_did::{Module, Call, Storage, Event<T>},
    Registrar: pallet_registrar::{Module, Call, Storage, Event<T>},
	TariffManager: pallet_tariff_manager::{Module, Call, Storage, Event<T>},
  }
);

frame_support::parameter_types! {
  pub const BlockHashCount: u64 = 250;
  pub BlockWeights: frame_system::limits::BlockWeights =
    frame_system::limits::BlockWeights::simple_max(1024);
}

frame_support::parameter_types! {
  pub const MinimumPeriod: u64 = 5;
}

impl frame_system::Config for Test {
    type BaseCallFilter = ();
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type Origin = Origin;
    type Index = u64;
    type BlockNumber = u64;
    type Call = Call;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    //type AccountId = u64;
    type AccountId = sp_core::sr25519::Public;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = Event;
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
}

impl pallet_timestamp::Config for Test {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = ();
}

impl pallet_session_payment::Config for Test {
    type Event = Event;
    type AuthorityId = pallet_session_payment::crypto::PaymentValidatorId;
}

impl pallet_charge_consent::Config for Test {
    type Event = Event;
}

impl pallet_registrar::Config for Test {
    type Event = Event;
}

impl pallet_tariff_manager::Config for Test {
	type Event = Event;
}

impl pallet_did::Config for Test {
    type Event = Event;
    type Public = <Signature as Verify>::Signer;
    type Signature = Signature;
    type Time = Timestamp;
}

impl frame_system::offchain::SigningTypes for Test {
    type Public = <Signature as Verify>::Signer;
    type Signature = Signature;
}

type Extrinsic = TestXt<Call, ()>;
type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

impl<LocalCall> frame_system::offchain::SendTransactionTypes<LocalCall> for Test
where
    Call: From<LocalCall>,
{
    type OverarchingCall = Call;
    type Extrinsic = Extrinsic;
}

impl<LocalCall> frame_system::offchain::CreateSignedTransaction<LocalCall> for Test
where
    Call: From<LocalCall>,
{
    fn create_transaction<C: frame_system::offchain::AppCrypto<Self::Public, Self::Signature>>(
        call: Call,
        _public: <Signature as Verify>::Signer,
        _account: AccountId,
        nonce: u64,
    ) -> Option<(Call, <Extrinsic as ExtrinsicT>::SignaturePayload)> {
        Some((call, (nonce, ())))
    }
}

use hex_literal::hex;
use sp_core::sr25519::Public;

fn register_new_usr(user: Public) {
    assert_ok!(SessionPayment::new_consent(
        Origin::signed(user),
        Vec::from("iban1"),
        Vec::from("bic_code1"),
		Vec::from("Signature"),
    ));
}

pub fn register_payment_validator(validator: Public) {
    let admin = Public::from_raw(hex!(
        "108fa7489d496834f3cbbc690798dbda53cf8edc6781672f706a031afcfb811f"
    ));
    if Registrar::organizations().contains(&admin) == false {
        assert_ok!(Registrar::create_organization(
            Origin::signed(admin),
            b"payment_validators".to_vec()
        ));
    }
    assert_ok!(Registrar::add_to_organization(
        Origin::signed(admin),
        validator
    ));
}

pub fn register_new_session(user: Public, charger: Public, session_id: H256) {
    if ChargeConsent::get_consent_from_session_id(session_id).is_none() {
        assert_ok!(ChargeConsent::new_consent_for_user(
            Origin::signed(user),
            charger,
            session_id
        ));
    }
}

#[test]
fn should_register_new_user() {
    new_test_ext().execute_with(|| {
        let user = Public::from_raw(hex!(
            "9a75da2249c660ca3c6bc5f7ff925ffbbbf5332fa09ab1e0540d748570c8ce27"
        ));
        register_new_usr(user);
    });
}

#[test]
fn should_be_allowed_to_pay() {
    new_test_ext().execute_with(|| {
        let user = Public::from_raw(hex!(
            "9a75da2249c660ca3c6bc5f7ff925ffbbbf5332fa09ab1e0540d748570c8ce27"
        ));
        register_new_usr(user);
        assert_eq!(SessionPayment::has_consent(&user), true);
    });
}

#[test]
fn should_not_be_allowed_to_pay() {
    new_test_ext().execute_with(|| {
        let user = Public::from_raw(hex!(
            "bec4ab0eaff1a0d710274b3648bc5b2253e2bdee293987123962688f08a5c317"
        ));
        register_new_usr(user);

        let user2 = Public::from_raw(hex!(
            "9a75da2249c660ca3c6bc5f7ff925ffbbbf5332fa09ab1e0540d748570c8ce27"
        ));

        assert_eq!(SessionPayment::has_consent(&user2),false);
    });
}

#[test]
fn should_process_payment_for_user_with_consent() {
    new_test_ext().execute_with(|| {
        let user = Public::from_raw(hex!(
            "bec4ab0eaff1a0d710274b3648bc5b2253e2bdee293987123962688f08a5c317"
        ));
        let charger = Public::from_raw(hex!(
            "f42bbe8f90ae3f9a1029a7bfaeca74fb5ca0c759a0d0476610c1eb4c60a40938"
        ));
        register_new_usr(user);
        let session_id = <Test as frame_system::Config>::Hashing::hash(&user);
        register_new_session(user, charger, session_id);

        assert_ok!(SessionPayment::process_payment(Origin::signed(charger), session_id, 1000));
    });
}

#[test]
fn should_not_process_payment_for_user_without_consent() {
    new_test_ext().execute_with(|| {
        let user = Public::from_raw(hex!(
            "9a75da2249c660ca3c6bc5f7ff925ffbbbf5332fa09ab1e0540d748570c8ce27"
        ));
        let hash = <Test as frame_system::Config>::Hashing::hash(&user);
        assert_err!(
            SessionPayment::process_payment(Origin::signed(user), hash, 1000),
            pallet_session_payment::Error::<Test>::NoConsentForPayment
        );
    });
}

#[test]
fn should_reject_payment_completion_from_unauthorized_account() {
    new_test_ext().execute_with(|| {
        let sender = Public::from_raw(hex!(
            "9a75da2249c660ca3c6bc5f7ff925ffbbbf5332fa09ab1e0540d748570c8ce27"
        ));
        let session_id = <Test as frame_system::Config>::Hashing::hash(&sender);
        assert_err!(
            SessionPayment::complete_payment(Origin::signed(sender), session_id),
            pallet_session_payment::Error::<Test>::NotRegisteredPaymentValidator
        );
    });
}

#[test]
fn should_reject_payment_completion_for_non_existent_payments() {
    new_test_ext().execute_with(|| {
        let sender = Public::from_raw(hex!(
            "54ac0c914b2d1552d4749276b0eb547b881486a8c224d3cf8207e9d2f9a91b79"
        ));
        register_payment_validator(sender);
        let session_id = <Test as frame_system::Config>::Hashing::hash(&sender);
        assert_err!(
            SessionPayment::complete_payment(Origin::signed(sender), session_id),
            pallet_session_payment::Error::<Test>::NonExistentPayment
        );
    });
}

#[test]
fn should_allow_payment_completion_from_authorized_account() {
    new_test_ext().execute_with(|| {
        let user = Public::from_raw(hex!(
            "40d5214f1b350475789a7541d0f13471f5ec5f41e765933b6c602115c87e5f79"
        ));
        let charger = Public::from_raw(hex!(
            "ce94587fd243e247cb144ff0f40d78a94b487c0170f03596ffd6366e7a9d5c0c"
        ));
        let offchain_worker = Public::from_raw(hex!(
            "54ac0c914b2d1552d4749276b0eb547b881486a8c224d3cf8207e9d2f9a91b79"
        ));
        register_new_usr(user);
        register_payment_validator(offchain_worker);
        
        let session_id = <Test as frame_system::Config>::Hashing::hash(&user);
        register_new_session(user, charger, session_id);

        assert_ok!(SessionPayment::process_payment(Origin::signed(charger), session_id, 1000));
        assert_eq!(SessionPayment::completed_payments(session_id).is_none(), true);
        assert_eq!(SessionPayment::pending_payments().len(), 1);
        assert_ok!(
            SessionPayment::complete_payment(Origin::signed(offchain_worker), session_id)
        );
        assert_eq!(SessionPayment::completed_payments(session_id).is_some(), true);
        assert_eq!(SessionPayment::pending_payments().len(), 0);
    });
}
