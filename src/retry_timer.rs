use std::time::{Duration, Instant};

/// A simple struct allowing to know when to retry.
/// The first try after a creation or reset is instant.
pub struct RetryTimer {
    last_retry: Instant,
    try_count: u32,
}

impl RetryTimer {
    pub fn new() -> Self {
        Self {
            last_retry: Instant::now(),
            try_count: 0,
        }
    }

    /// Return the total time that should be waited to next try, since the last failure
    pub fn seconds_to_wait(&self) -> u64 {
        if self.try_count == 0 {
            0
        } else {
            let mut seconds_to_wait: u64 = 30;
            for _ in 1..self.try_count {
                seconds_to_wait = seconds_to_wait.overflowing_mul(3).0;
                if seconds_to_wait > 900 {
                    // retry every 15 minutes no matter what
                    seconds_to_wait = 900;
                    break;
                }
            }
            seconds_to_wait
        }
    }

    /// Return true if a new retry should be done
    pub fn should_retry(&self) -> bool {
        let now = Instant::now();

        now.duration_since(self.last_retry.clone()) > Duration::from_secs(self.seconds_to_wait())
    }

    /// Should be called once a (re)try has succeded, so the first next try is immediate
    pub fn post_success(&mut self) {
        self.last_retry = Instant::now();
        self.try_count = 0;
    }

    /// Should be called once a (re)try has been performed and failed
    pub fn post_failure(&mut self) {
        // Do not increase retry timer if it was retried sonner than planned for some reason
        if self.should_retry() {
            self.try_count = self.try_count.saturating_add(1);
        }
        self.last_retry = Instant::now();
    }
}
