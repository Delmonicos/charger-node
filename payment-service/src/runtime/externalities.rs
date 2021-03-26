use crate::runtime::offchain;
use crate::{
    api::{ChargeStatus, PaymentApi},
    mock::MockPayment,
};
use std::sync::{Arc, Mutex};

pub trait Externalities: Send {
    fn start_charge(&mut self) -> bool;
    fn get_current_charge_status(&mut self) -> offchain::ChargeStatus;
}
pub struct PaymentExternalities<T>
where
    T: PaymentApi,
{
    api: Arc<Mutex<T>>,
}

impl PaymentExternalities<MockPayment> {
    pub fn new(api: Arc<Mutex<MockPayment>>) -> PaymentExternalities<MockPayment> {
        PaymentExternalities { api }
    }
}

impl<T: PaymentApi + Send> Externalities for PaymentExternalities<T> {
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
    pub struct PaymentExt(Box<dyn Externalities>);
}

impl PaymentExt {
    /// Create a new instance of `Self`.
    pub fn new<O: Externalities + 'static>(charger: O) -> Self {
        Self(Box::new(charger))
    }
}
