// Creating mock runtime here

use frame_support::{impl_outer_origin, parameter_types, weights::Weight};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    Perbill,
};
use crate::pallet::*;

impl_outer_origin! {
    pub enum Origin for Test where system = frame_system {}
}

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const MaximumBlockWeight: Weight = 1024;
    pub const MaximumBlockLength: u32 = 2 * 1024;
    pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
}

// For testing the pallet, we construct most of a mock runtime. This means
// first constructing a configuration type (`Test`) which `impl`s each of the
// configuration traits of pallets we want to use.
#[derive(Clone, Eq, PartialEq)]
pub struct Test;

impl system::Config for Test {
    type BaseCallFilter = ();
    type Origin = Origin;
    type Call = ();
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = ();
    type BlockHashCount = BlockHashCount;
    type DbWeight = ();
    type BlockLength = ();
	type BlockWeights = ();
    type Version = ();
	type PalletInfo = Self;
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
	type SS58Prefix = ();
    type OnSetCode = ();
}

parameter_types! {
    pub const MaxCommodities: u128 = 5;
    pub const MaxCommoditiesPerUser: u64 = 2;
}

impl Config for Test {
    type Event = ();
    type CommodityAdmin = frame_system::EnsureRoot<Self::AccountId>;
    type CommodityInfo = Vec<u8>;
    type CommodityLimit = MaxCommodities;
    type UserCommodityLimit = MaxCommoditiesPerUser;
}

// system under test
pub type Commodities = Pallet<Test>;
// This function basically just builds a genesis storage key/value store according to
// our desired mockup.
pub fn new_test_ext() -> sp_io::TestExternalities {
    system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap()
        .into()
}

impl frame_support::traits::PalletInfo for Test {
	fn index<P: 'static>() -> Option<usize> {
		Some(0)
	}
	fn name<P: 'static>() -> Option<&'static str> {
		return Some("Commodities")
	}
}
