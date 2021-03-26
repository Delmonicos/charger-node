use anyhow::{anyhow, Result};
use log::{debug, info};
use rand::prelude::*;
use std::{
    ops::Add,
    time::{Duration, Instant},
};

use crate::api::*;

pub struct MockPayment {
    current_session_end_timestamp: Option<Instant>,
    min_time: u64,
    max_time: u64,
}

impl MockPayment {
    pub fn new() -> MockPayment {
        MockPayment {
            current_session_end_timestamp: None,
            min_time: 20,
            max_time: 60,
        }
    }
}

impl PaymentApi for MockPayment {
    fn start_new_charge(&mut self) -> Result<()> {
        if self.current_session_end_timestamp.is_some() {
            return Err(anyhow!("This charger already has an active session"));
        }
        let mut rng = rand::thread_rng();
        let end_at = std::time::Instant::now().add(Duration::from_secs(
            rng.gen_range(self.min_time..self.max_time),
        ));
        debug!("New charge session started, end at {:?}", end_at);
        self.current_session_end_timestamp = Some(end_at);
        Ok(())
    }

    fn get_current_charge_status(&mut self) -> Result<ChargeStatus> {
        debug!("Get charge status");
        let status = match self.current_session_end_timestamp {
            None => ChargeStatus::NotFound,
            Some(end_at) if Instant::now() > end_at => {
                self.current_session_end_timestamp = None;
                let mut rng = rand::thread_rng();
                let kwh = rng.gen_range(100..5000);
                info!("Charge is ended, kwh: {}", kwh);
                ChargeStatus::Ended { kwh }
            }
            _ => ChargeStatus::Active,
        };
        Ok(status)
    }
}

#[cfg(test)]
mod test {
    use crate::api::{ChargeStatus, PaymentApi};
    use crate::mock::MockPayment;

    #[test]
    fn should_create_new_session() {
        let mut charger_api = MockPayment::new();

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
    fn should_end_session() {
        let mut charger_api = MockPayment {
            current_session_end_timestamp: None,
            min_time: 1,
            max_time: 2,
        };

        // Start new charge
        charger_api
            .start_new_charge()
            .expect("Cannot start new charge");

        // Check status after 2s
        std::thread::sleep(std::time::Duration::from_secs(2));

        let status_after = charger_api
            .get_current_charge_status()
            .expect("Cannot get charge status");
        assert!(matches!(status_after, ChargeStatus::Ended { .. }));
    }
}
