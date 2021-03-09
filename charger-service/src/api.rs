use anyhow::Result;

#[derive(Debug, PartialEq)]
pub enum ChargeStatus {
    NotFound,
    Active,
    Ended { kwh: u64 },
}

pub trait ChargerApi {
    /// Start a new charge session
    fn start_new_charge(&mut self) -> Result<u64>;

    /// Get charge session status
    fn get_charge_status(&mut self, id: u64) -> Result<ChargeStatus>;
}
