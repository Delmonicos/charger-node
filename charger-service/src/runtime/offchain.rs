use sp_externalities::ExternalitiesExt;
use codec::{Decode, Encode};
use sp_runtime_interface::pass_by::PassByCodec;

#[cfg(feature = "std")]
use super::externalities::ChargerExt;

#[derive(Encode, Decode, PassByCodec)]
pub enum ChargeStatus {
    NoCharge,
    Active,
    Ended { kwh: u64 }
}

#[sp_runtime_interface::runtime_interface]
pub trait Api {
    fn start_charge(&mut self) -> bool {
        return self
            .extension::<ChargerExt>()
            .expect("no extension")
            .start_charge();
    }

    fn get_current_charge_status(&mut self) -> ChargeStatus {
        return self
            .extension::<ChargerExt>()
            .expect("no extension")
            .get_current_charge_status();
    }
}
