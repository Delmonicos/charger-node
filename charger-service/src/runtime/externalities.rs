use crate::{
    api::{ChargeStatus, ChargerApi},
    mock::MockCharger,
};
use std::sync::{Arc, Mutex};

pub trait Externalities: Send {
    fn start_charge(&mut self) -> bool;
    fn get_current_charge_status(&mut self) -> bool;
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

    fn get_current_charge_status(&mut self) -> bool {
        match self.api.lock().unwrap().get_current_charge_status() {
            Ok(ChargeStatus::Ended { kwh: _ }) => true,
            _ => false,
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
