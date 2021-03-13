use anyhow::Result;
use log::{debug, info};
use std::time::{Duration, Instant};

use crate::api::*;

pub struct MockCharger {
    active_charge_since: Option<Instant>,
}

impl MockCharger {
    pub fn new() -> MockCharger {
        MockCharger {
            active_charge_since: None,
        }
    }
}

impl ChargerApi for MockCharger {
    fn start_new_charge(&mut self) -> Result<()> {
        let now = std::time::Instant::now();
        debug!("New charge session started");
        self.active_charge_since = Some(now);
        Ok(())
    }

    fn get_current_charge_status(&mut self) -> Result<ChargeStatus> {
        debug!("Get charge status");
        let status = match self.active_charge_since {
            None => ChargeStatus::NotFound,
            Some(created_at)
                if Instant::now().duration_since(created_at.clone()) > Duration::from_secs(20) =>
            {
                self.active_charge_since = None;
                info!("Charge is ended!");
                ChargeStatus::Ended { kwh: 999 }
            }
            _ => ChargeStatus::Active,
        };
        Ok(status)
    }
}

#[cfg(test)]
mod test {
    use crate::api::{ChargeStatus, ChargerApi};
    use crate::mock::MockCharger;

    #[test]
    fn should_create_new_session() {
        let mut charger_api = MockCharger::new();

        // Check status before
        let status_before = charger_api
            .get_current_charge_status()
            .expect("Cannot get charge status");
        assert_eq!(status_before, ChargeStatus::NotFound);

        // Start new charge
        assert_eq!(charger_api.start_new_charge().is_ok(), true);

        // Check status after
        let status_after = charger_api
            .get_current_charge_status()
            .expect("Cannot get charge status");
        assert_eq!(status_after, ChargeStatus::Active);
    }

    #[test]
    fn should_end_session_after_20s() {
        let mut charger_api = MockCharger::new();

        // Start new charge
        charger_api
            .start_new_charge()
            .expect("Cannot start new charge");

        // Check status after 21s
        std::thread::sleep(std::time::Duration::from_secs(21));

        let status_after = charger_api
            .get_current_charge_status()
            .expect("Cannot get charge status");
        assert_eq!(status_after, ChargeStatus::Ended { kwh: 999 });
    }
}
