use crate as pallet_charge_consent;

use frame_support::assert_ok;
use sp_core::{sr25519::Signature, H256};
use sp_io::TestExternalities;

use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, Hash, IdentityLookup, Verify},
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
    ChargeConsent: pallet_charge_consent::{Module, Call, Storage, Event<T>},
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

impl pallet_charge_consent::Config for Test {
    type Event = Event;
}

impl frame_system::offchain::SigningTypes for Test {
    type Public = <Signature as Verify>::Signer;
    type Signature = Signature;
}

use hex_literal::hex;
use sp_core::sr25519::Public;

#[test]
fn should_register_new_consent() {
    new_test_ext().execute_with(|| {
        let user = Public::from_raw(hex!(
            "9a75da2249c660ca3c6bc5f7ff925ffbbbf5332fa09ab1e0540d748570c8ce27"
        ));
        let charger = Public::from_raw(hex!(
            "bec4ab0eaff1a0d710274b3648bc5b2253e2bdee293987123962688f08a5c317"
        ));
        let session_id = <Test as frame_system::Config>::Hashing::hash(&user);
        assert_ok!(ChargeConsent::new_consent_for_user(
            Origin::signed(user),
            charger,
            session_id
        ));
    });
}

#[test]
fn should_find_consent_from_id() {
    new_test_ext().execute_with(|| {
        let user = Public::from_raw(hex!(
            "9a75da2249c660ca3c6bc5f7ff925ffbbbf5332fa09ab1e0540d748570c8ce27"
        ));
        let charger = Public::from_raw(hex!(
            "bec4ab0eaff1a0d710274b3648bc5b2253e2bdee293987123962688f08a5c317"
        ));
        let session_id = <Test as frame_system::Config>::Hashing::hash(&user);
        assert_ok!(ChargeConsent::new_consent_for_user(Origin::signed(user), charger, session_id));
        assert!(ChargeConsent::get_consent_from_session_id(session_id).is_some());
    });
}

#[test]
fn should_not_find_consent_from_id() {
    new_test_ext().execute_with(|| {
        let user = Public::from_raw(hex!(
            "9a75da2249c660ca3c6bc5f7ff925ffbbbf5332fa09ab1e0540d748570c8ce27"
        ));
        let session_id = <Test as frame_system::Config>::Hashing::hash(&user);
        assert!(ChargeConsent::get_consent_from_session_id(session_id).is_none());
    });
}
