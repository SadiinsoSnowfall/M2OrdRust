use std::cmp::Ordering;

use super::Clock;


#[derive(Debug, Eq, PartialOrd)]
pub struct Job {
	pub id: u32,
	pub nodes: u32,

	pub submit_time: Clock,
	pub schedule_time: Clock,

	pub requested_run_time: Clock,
	pub expected_end: Clock,

	pub scheduled: bool,

	pub run_time: Clock,
}

impl Job {
	pub fn new(id: u32, nodes: u32, submit_time: Clock, run_time: Clock, requested_run_time: Clock) -> Job {
		Job {
			id,
			nodes,
			requested_run_time,
			run_time,
			submit_time,
			scheduled: false,
			schedule_time: 0,
			expected_end: 0,
		}
	}

	pub fn set_scheduled(&mut self, clock: Clock) {
		self.scheduled = true;
		self.schedule_time = clock;
		self.expected_end = clock + self.requested_run_time;
	}

	pub fn wait_time(&self) -> Clock {
		assert!(self.scheduled);
		self.schedule_time - self.submit_time
	}

	pub fn wait_time_from(&self, clock: Clock) -> Clock {
		clock - self.submit_time
	}
}

impl PartialEq for Job {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Ord for Job {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}
