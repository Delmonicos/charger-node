use log::{debug, info};
use anyhow::Result;
use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::api::*;

pub struct MockCharger {
    active_charges: HashMap<u64, Instant>,
    next_id: u64,
}

impl MockCharger {
    pub fn new() -> MockCharger {
        MockCharger {
            active_charges: HashMap::new(),
            next_id: 0,
        }
    }
}

impl ChargerApi for MockCharger {
    fn start_new_charge(&mut self) -> Result<u64> {
        let now = std::time::Instant::now();
        let id = self.next_id;
        self.next_id = id + 1;
        self.active_charges.insert(id, now);
        info!("New charge session  with id {}", id);
        Ok(id)
    }

    fn get_charge_status(&mut self, id: u64) -> Result<ChargeStatus> {
        debug!("Get charge status for id {}", id);
        let status = match self.active_charges.get(&id) {
            None => ChargeStatus::NotFound,
            Some(created_at)
                if Instant::now().duration_since(created_at.clone()) > Duration::from_secs(10) =>
            {
                self.active_charges.remove(&id);
                info!("Charge {} is ended!", id);
                ChargeStatus::Ended { kwh: 999 }
            }
            _ => ChargeStatus::Active,
        };
        Ok(status)
    }
}

#[cfg(test)]
mod test {
    use crate::api::{ChargerApi, ChargeStatus};
    use crate::mock::MockCharger;

    #[test]
    fn should_create_new_session() {
        let mut charger_api = MockCharger::new();

        // Check status before
        let status_before = charger_api
            .get_charge_status(0)
            .expect("Cannot get charge status");
        assert_eq!(status_before, ChargeStatus::NotFound);

        // Start new charge
        let charge_id = charger_api
            .start_new_charge()
            .expect("Cannot start new charge");
        assert_eq!(charge_id, 0);

        // Check status after
        let status_after = charger_api
            .get_charge_status(0)
            .expect("Cannot get charge status");
        assert_eq!(status_after, ChargeStatus::Active);
    }

    #[test]
    fn should_end_session_after_10s() {
        let mut charger_api = MockCharger::new();

        // Start new charge
        let charge_id = charger_api
            .start_new_charge()
            .expect("Cannot start new charge");

        // Check status after 10s
        std::thread::sleep(std::time::Duration::from_secs(11));

        let status_after = charger_api
            .get_charge_status(charge_id)
            .expect("Cannot get charge status");
        assert_eq!(status_after, ChargeStatus::Ended { kwh: 999 });
    }
}
