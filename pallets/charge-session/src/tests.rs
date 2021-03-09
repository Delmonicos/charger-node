use crate as pallet_charge_session;

use sp_io::TestExternalities;
use sp_core::H256;
use sp_runtime::{traits::{BlakeTwo256, IdentityLookup}, testing::Header};
use frame_support::{assert_ok, assert_err};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

pub fn new_test_ext() -> TestExternalities {
  frame_system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
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
  type AccountId = u64;
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
}

#[test]
fn should_create_new_request() {
  new_test_ext().execute_with(|| {
    Timestamp::set_timestamp(999);

    let current_request = ChargeSession::user_requests(1);
    assert_eq!(current_request, None);

    assert_ok!(ChargeSession::new_request(Origin::signed(1), 2));

    let current_request = ChargeSession::user_requests(1).unwrap();
    assert_eq!(current_request.charger_id, 2);
    assert_eq!(current_request.created_at, 999);
  });
}

#[test]
fn should_replace_existing_request() {
  new_test_ext().execute_with(|| {
    Timestamp::set_timestamp(999);
    assert_ok!(ChargeSession::new_request(Origin::signed(1), 2));

    let current_request = ChargeSession::user_requests(1).unwrap();
    assert_eq!(current_request.charger_id, 2);
    assert_eq!(current_request.created_at, 999);

    Timestamp::set_timestamp(1000);
    assert_ok!(ChargeSession::new_request(Origin::signed(1), 3));
    
    let current_request = ChargeSession::user_requests(1).unwrap();
    assert_eq!(current_request.charger_id, 3);
    assert_eq!(current_request.created_at, 1000);
  });
}

#[test]
fn should_start_a_new_session() {
  new_test_ext().execute_with(|| {
    Timestamp::set_timestamp(999);
    assert!(ChargeSession::active_sessions(2).is_none());
    assert_ok!(ChargeSession::new_request(Origin::signed(1), 2));
    
    assert_ok!(ChargeSession::start_session(Origin::signed(2), 1));
    assert!(ChargeSession::user_requests(1).is_none());
    
    let session = ChargeSession::active_sessions(2).unwrap();
    
    assert_eq!(session.user_id, 1);
    assert_eq!(session.started_at, 999);
  });
}

#[test]
fn should_not_start_unrequested_session() {
  new_test_ext().execute_with(|| {
    assert_err!(ChargeSession::start_session(Origin::signed(2), 1), pallet_charge_session::Error::<Test>::NoChargingRequest);
    assert!(ChargeSession::active_sessions(2).is_none());
  });
}

#[test]
fn should_not_start_twice() {
  new_test_ext().execute_with(|| {
    assert_ok!(ChargeSession::new_request(Origin::signed(1), 2));
    assert_ok!(ChargeSession::start_session(Origin::signed(2), 1));
    assert_err!(ChargeSession::start_session(Origin::signed(2), 1), pallet_charge_session::Error::<Test>::NoChargingRequest);
  });
}

#[test]
fn should_not_take_request_from_another_charger() {
  new_test_ext().execute_with(|| {
    assert_ok!(ChargeSession::new_request(Origin::signed(1), 3));
    assert_err!(ChargeSession::start_session(Origin::signed(2), 1), pallet_charge_session::Error::<Test>::NoChargingRequest);
  });
}

#[test]
fn should_end_an_active_session() {
  new_test_ext().execute_with(|| {
    assert_ok!(ChargeSession::new_request(Origin::signed(1), 2));
    assert_ok!(ChargeSession::start_session(Origin::signed(2), 1));
    assert_ok!(ChargeSession::end_session(Origin::signed(2), 1));

    assert!(ChargeSession::active_sessions(2).is_none());
  });
}

#[test]
fn should_not_end_a_session_from_another_charger() {
  new_test_ext().execute_with(|| {
    assert_ok!(ChargeSession::new_request(Origin::signed(1), 2));
    assert_ok!(ChargeSession::start_session(Origin::signed(2), 1));
    assert_err!(ChargeSession::end_session(Origin::signed(3), 1,), pallet_charge_session::Error::<Test>::NoChargingSession);

    assert!(ChargeSession::active_sessions(2).is_some());
  });
}