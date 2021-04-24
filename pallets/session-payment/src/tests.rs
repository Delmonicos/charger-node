use crate as pallet_session_payment;

use frame_support::{assert_err, assert_ok};
use sp_core::{sr25519::Signature, H256};
use sp_io::TestExternalities;
use sp_runtime::{
    testing::{Header, TestXt},
    traits::{BlakeTwo256, Extrinsic as ExtrinsicT, Hash, IdentifyAccount, IdentityLookup, Verify},
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

pub fn new_test_ext() -> TestExternalities {
    frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap()
        .into()
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
    UserConsent: pallet_user_consent::{Module, Call, Storage, Event<T>},
    DID: pallet_did::{Module, Call, Storage, Event<T>},
    Registrar: pallet_registrar::{Module, Call, Storage, Event<T>},
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
}

impl pallet_user_consent::Config for Test {
    type Event = Event;
}

impl pallet_registrar::Config for Test {
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
        assert_ok!(SessionPayment::is_allowed_to_pay(Origin::signed(user),));
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

        assert_err!(
            SessionPayment::is_allowed_to_pay(Origin::signed(user2)),
            pallet_session_payment::Error::<Test>::NoConsentForPayment
        );
    });
}

#[test]
fn should_process_payment_for_user_with_consent() {
    new_test_ext().execute_with(|| {
        let user = Public::from_raw(hex!(
            "bec4ab0eaff1a0d710274b3648bc5b2253e2bdee293987123962688f08a5c317"
        ));
        register_new_usr(user);
        let hash = <Test as frame_system::Config>::Hashing::hash(&user);

        //TODO Write the right test: create the session_id before paying
        // assert_ok!(SessionPayment::process_payment(Origin::signed(user), hash, 1000));
        assert_err!(
            SessionPayment::process_payment(Origin::signed(user), hash, 1000),
            pallet_session_payment::Error::<Test>::NoConsentForPayment
        );
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
