use std::time::{Duration, SystemTime};
use rand::Rng;


const DEFAULT_MIN_FREQUENCY: Duration = Duration::from_millis(10);
const DEFAULT_MAX_FREQUENCY: Duration = Duration::from_millis(100);

#[derive(Clone, Copy)]
pub struct ElectionTimer {
    pub min_freq: Duration,
    pub max_freq: Duration,
    pub next_election: SystemTime
}

impl ElectionTimer {
    pub fn new(min_f: Duration, max_f: Duration) -> Self {
        let mut e = ElectionTimer{
            min_freq: min_f,
            max_freq: max_f,
            next_election: SystemTime::now()
        };
        e.renew_election_timeout();
        e
    }

    // Reset to random time  between [min_freq, max_freq] into the future.
    pub fn renew_election_timeout(&mut self) {
        let random_duration = self.min_freq.saturating_add(
            rand::thread_rng().gen_range(Duration::from_nanos(0)..self.max_freq.saturating_sub(self.min_freq)),
        );

        self.next_election = SystemTime::now().checked_add(random_duration).unwrap_or_else(|| {
            panic!("Failed to add duration to SystemTime, resulting time is too large to represent!");
        });
    }

    pub fn until_next_election(self) -> Duration {
        match self.next_election.duration_since(SystemTime::now()) {
            Ok(dur) => dur,

            // Error can occur if next election time is in the past. Return 0 in this case.
            Err(_) => Duration::new(0, 0)
        }
    }
}

impl Default for ElectionTimer {
    fn default() -> Self {
        Self::new(DEFAULT_MIN_FREQUENCY, DEFAULT_MAX_FREQUENCY)
    }
}
