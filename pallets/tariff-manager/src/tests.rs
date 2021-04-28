use crate as pallet_tariff_manager;

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

impl pallet_tariff_manager::Config for Test {
    type Event = Event;
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
fn should_register_new_tariff() {
    new_test_ext().execute_with(|| {
        let user = Public::from_raw(hex!(
            "9a75da2249c660ca3c6bc5f7ff925ffbbbf5332fa09ab1e0540d748570c8ce27"
        ));
        let smart_contract = Public::from_raw(hex!(
            "8ff2ad81f54d09f503d8abc01fc6e50d34e5ad8c5784a284faf453475a328ca9"
        ));
        assert_ok!(TariffManager::new_tariff(
            Origin::signed(user),
            Vec::from("fixed_price"),
            smart_contract,
        ));
    });
}
