use crate::mock::MockCharger;
use crate::api::ChargerApi;

pub trait Externalities: Send {
	fn is_charger(&self) -> bool;
	fn start_charge(&mut self) -> bool;
}
pub struct ChargerExternalities {
	api: MockCharger
}

impl ChargerExternalities {
	pub fn new() -> ChargerExternalities {
		ChargerExternalities {
			api: MockCharger::new()
		}
	}
}

impl Externalities for ChargerExternalities {
  fn is_charger(&self) -> bool {
    return true;
  }

	fn start_charge(&mut self) -> bool {
    return self.api.start_new_charge().is_ok();
  }
}

sp_externalities::decl_extension! {
	pub struct ChargerExt(Box<dyn Externalities>);
}

impl ChargerExt {
	/// Create a new instance of `Self`.
	pub fn new<O: Externalities + 'static>(charger: O) -> Self {
		Self(Box::new(charger))
	}
}