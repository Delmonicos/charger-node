use sp_externalities::ExternalitiesExt;

#[cfg(feature = "std")]
use super::externalities::ChargerExt;

#[sp_runtime_interface::runtime_interface]
pub trait Api {
    fn start_charge(&mut self) -> bool {
        return self
            .extension::<ChargerExt>()
            .expect("no extension")
            .start_charge();
    }

    fn get_current_charge_status(&mut self) -> bool {
        // return true if charge session is active
        return self
            .extension::<ChargerExt>()
            .expect("no extension")
            .get_current_charge_status();
    }
}
