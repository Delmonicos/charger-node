use sp_externalities::ExternalitiesExt;

#[cfg(feature = "std")]
use super::externalities::ChargerExt;

#[sp_runtime_interface::runtime_interface]
pub trait Api {

  fn is_charger(&mut self) -> bool {
    return self.extension::<ChargerExt>().expect("no extension").is_charger();
  }

  fn start_charge(&mut self) -> bool {
    return self.extension::<ChargerExt>().expect("no extension").start_charge();
  }

}
