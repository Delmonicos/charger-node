use crate::{
    api::{ChargeStatus, ChargerApi},
    mock::MockCharger,
};
use std::sync::{Arc, Mutex};
use crate::runtime::offchain;

pub trait Externalities: Send {
    fn start_charge(&mut self) -> bool;
    fn get_current_charge_status(&mut self) -> offchain::ChargeStatus;
}
pub struct ChargerExternalities<T>
where
    T: ChargerApi,
{
    api: Arc<Mutex<T>>,
}

impl ChargerExternalities<MockCharger> {
    pub fn new(api: Arc<Mutex<MockCharger>>) -> ChargerExternalities<MockCharger> {
        ChargerExternalities { api }
    }
}

impl<T: ChargerApi + Send> Externalities for ChargerExternalities<T> {
    fn start_charge(&mut self) -> bool {
        return self.api.lock().unwrap().start_new_charge().is_ok();
    }

    fn get_current_charge_status(&mut self) -> offchain::ChargeStatus {
        match self.api.lock().unwrap().get_current_charge_status() {
            Ok(ChargeStatus::Ended { kwh }) => offchain::ChargeStatus::Ended { kwh },
            Ok(ChargeStatus::Active) => offchain::ChargeStatus::Active,
            _ => offchain::ChargeStatus::NoCharge,
        }
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
