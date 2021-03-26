use anyhow::Result;

#[derive(Debug, PartialEq)]
pub enum ChargeStatus {
    NotFound,
    Active,
    Ended { kwh: u64 },
}

pub trait PaymentApi {
    /// Start a new charge session
    fn start_new_charge(&mut self) -> Result<()>;

    /// Get charge session status
    fn get_current_charge_status(&mut self) -> Result<ChargeStatus>;
}
