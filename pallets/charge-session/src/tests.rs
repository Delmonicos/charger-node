use crate as pallet_charge_session;

use frame_support::{assert_err, assert_ok};
use sp_core::{sr25519::Signature, H256};
use sp_io::TestExternalities;
use sp_runtime::{
    testing::{Header, TestXt},
    traits::{BlakeTwo256, Extrinsic as ExtrinsicT, IdentifyAccount, IdentityLookup, Verify},
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
    ChargeSession: pallet_charge_session::{Module, Call, Storage, Event<T>},
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

impl pallet_charge_session::Config for Test {
    type Event = Event;
    type AutorityId = pallet_charge_session::crypto::ChargerId;
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

#[test]
fn should_create_new_request() {
    new_test_ext().execute_with(|| {
        let user = Public::from_raw(hex!(
            "bec4ab0eaff1a0d710274b3648bc5b2253e2bdee293987123962688f08a5c317"
        ));
        let charger = Public::from_raw(hex!(
            "9a75da2249c660ca3c6bc5f7ff925ffbbbf5332fa09ab1e0540d748570c8ce27"
        ));

        Timestamp::set_timestamp(999);

        let current_request = ChargeSession::user_requests(user);
        assert_eq!(current_request, None);

        assert_ok!(ChargeSession::new_request(Origin::signed(user), charger));

        let current_request = ChargeSession::user_requests(charger).unwrap();
        assert_eq!(current_request.user_id, user);
        assert_eq!(current_request.created_at, 999);
    });
}

#[test]
fn should_start_a_new_session() {
    new_test_ext().execute_with(|| {
        let user = Public::from_raw(hex!(
            "bec4ab0eaff1a0d710274b3648bc5b2253e2bdee293987123962688f08a5c317"
        ));
        let charger = Public::from_raw(hex!(
            "44ce5dedab4604c5df7d46ebd146ff5773bfcd975f7203e4cbac45149593a865"
        ));

        Timestamp::set_timestamp(999);
        assert!(ChargeSession::active_sessions(charger).is_none());
        assert_ok!(ChargeSession::new_request(Origin::signed(user), charger));

        assert_ok!(ChargeSession::start_session(Origin::signed(charger), user));
        assert!(ChargeSession::user_requests(charger).is_none());

        let session = ChargeSession::active_sessions(charger).unwrap();

        assert_eq!(session.user_id, user);
        assert_eq!(session.started_at, 999);
    });
}

#[test]
fn should_not_start_unrequested_session() {
    new_test_ext().execute_with(|| {
        let user = Public::from_raw(hex!(
            "bec4ab0eaff1a0d710274b3648bc5b2253e2bdee293987123962688f08a5c317"
        ));
        let charger = Public::from_raw(hex!(
            "44ce5dedab4604c5df7d46ebd146ff5773bfcd975f7203e4cbac45149593a865"
        ));

        assert_err!(
            ChargeSession::start_session(Origin::signed(charger), user),
            pallet_charge_session::Error::<Test>::NoChargingRequest
        );
        assert!(ChargeSession::active_sessions(charger).is_none());
    });
}

#[test]
fn should_not_start_twice() {
    new_test_ext().execute_with(|| {
        let user = Public::from_raw(hex!(
            "bec4ab0eaff1a0d710274b3648bc5b2253e2bdee293987123962688f08a5c317"
        ));
        let charger = Public::from_raw(hex!(
            "44ce5dedab4604c5df7d46ebd146ff5773bfcd975f7203e4cbac45149593a865"
        ));

        assert_ok!(ChargeSession::new_request(Origin::signed(user), charger));
        assert_ok!(ChargeSession::start_session(Origin::signed(charger), user));
        assert_err!(
            ChargeSession::start_session(Origin::signed(charger), user),
            pallet_charge_session::Error::<Test>::NoChargingRequest
        );
    });
}

#[test]
fn should_not_take_request_from_another_charger() {
    new_test_ext().execute_with(|| {
        let user = Public::from_raw(hex!(
            "bec4ab0eaff1a0d710274b3648bc5b2253e2bdee293987123962688f08a5c317"
        ));
        let charger_1 = Public::from_raw(hex!(
            "44ce5dedab4604c5df7d46ebd146ff5773bfcd975f7203e4cbac45149593a865"
        ));
        let charger_2 = Public::from_raw(hex!(
            "e6687af66d6b3a191061c519033b50d86907eaa4c7961ed416a5dc3042346036"
        ));

        assert_ok!(ChargeSession::new_request(Origin::signed(user), charger_2));
        assert_err!(
            ChargeSession::start_session(Origin::signed(charger_1), user),
            pallet_charge_session::Error::<Test>::NoChargingRequest
        );
    });
}

#[test]
fn should_end_an_active_session() {
    new_test_ext().execute_with(|| {
        let user = Public::from_raw(hex!(
            "bec4ab0eaff1a0d710274b3648bc5b2253e2bdee293987123962688f08a5c317"
        ));
        let charger = Public::from_raw(hex!(
            "e6687af66d6b3a191061c519033b50d86907eaa4c7961ed416a5dc3042346036"
        ));

        assert_ok!(ChargeSession::new_request(Origin::signed(user), charger));
        assert_ok!(ChargeSession::start_session(Origin::signed(charger), user));
        assert_ok!(ChargeSession::end_session(Origin::signed(charger), user, 99));

        assert!(ChargeSession::active_sessions(charger).is_none());
    });
}

#[test]
fn should_not_end_a_session_from_another_charger() {
    new_test_ext().execute_with(|| {
        let user = Public::from_raw(hex!(
            "bec4ab0eaff1a0d710274b3648bc5b2253e2bdee293987123962688f08a5c317"
        ));
        let charger_1 = Public::from_raw(hex!(
            "44ce5dedab4604c5df7d46ebd146ff5773bfcd975f7203e4cbac45149593a865"
        ));
        let charger_2 = Public::from_raw(hex!(
            "e6687af66d6b3a191061c519033b50d86907eaa4c7961ed416a5dc3042346036"
        ));

        assert_ok!(ChargeSession::new_request(Origin::signed(user), charger_1));
        assert_ok!(ChargeSession::start_session(
            Origin::signed(charger_1),
            user
        ));
        assert_err!(
            ChargeSession::end_session(Origin::signed(charger_2), user, 99),
            pallet_charge_session::Error::<Test>::NoChargingSession
        );

        assert!(ChargeSession::active_sessions(charger_1).is_some());
    });
}

#[test]
fn should_reject_new_request_if_request_already_exists() {
    new_test_ext().execute_with(|| {
        let user_1 = Public::from_raw(hex!(
            "bec4ab0eaff1a0d710274b3648bc5b2253e2bdee293987123962688f08a5c317"
        ));
        let user_2 = Public::from_raw(hex!(
            "44ce5dedab4604c5df7d46ebd146ff5773bfcd975f7203e4cbac45149593a865"
        ));
        let charger= Public::from_raw(hex!(
            "e6687af66d6b3a191061c519033b50d86907eaa4c7961ed416a5dc3042346036"
        ));

        assert_ok!(ChargeSession::new_request(Origin::signed(user_1), charger));
        assert_err!(
            ChargeSession::new_request(Origin::signed(user_2), charger),
            pallet_charge_session::Error::<Test>::ChargerIsBusy
        );
    });
}

#[test]
fn should_reject_new_request_if_charge_is_active() {
    new_test_ext().execute_with(|| {
        let user_1 = Public::from_raw(hex!(
            "bec4ab0eaff1a0d710274b3648bc5b2253e2bdee293987123962688f08a5c317"
        ));
        let user_2 = Public::from_raw(hex!(
            "44ce5dedab4604c5df7d46ebd146ff5773bfcd975f7203e4cbac45149593a865"
        ));
        let charger= Public::from_raw(hex!(
            "e6687af66d6b3a191061c519033b50d86907eaa4c7961ed416a5dc3042346036"
        ));

        assert_ok!(ChargeSession::new_request(Origin::signed(user_1), charger));
        assert_ok!(ChargeSession::start_session(Origin::signed(charger), user_1));
        assert_err!(
            ChargeSession::new_request(Origin::signed(user_2), charger),
            pallet_charge_session::Error::<Test>::ChargerIsBusy
        );
    });
}

#[test]
fn should_chain_two_sessions() {
    new_test_ext().execute_with(|| {
        let user_1 = Public::from_raw(hex!(
            "bec4ab0eaff1a0d710274b3648bc5b2253e2bdee293987123962688f08a5c317"
        ));
        let user_2 = Public::from_raw(hex!(
            "44ce5dedab4604c5df7d46ebd146ff5773bfcd975f7203e4cbac45149593a865"
        ));
        let charger= Public::from_raw(hex!(
            "e6687af66d6b3a191061c519033b50d86907eaa4c7961ed416a5dc3042346036"
        ));

        assert_ok!(ChargeSession::new_request(Origin::signed(user_1), charger));
        assert_ok!(ChargeSession::start_session(Origin::signed(charger), user_1));
        assert_ok!(ChargeSession::end_session(Origin::signed(charger), user_1, 99));

        assert_ok!(ChargeSession::new_request(Origin::signed(user_2), charger));
        assert_ok!(ChargeSession::start_session(Origin::signed(charger), user_2));
        assert_ok!(ChargeSession::end_session(Origin::signed(charger), user_2, 99));
    });
}
