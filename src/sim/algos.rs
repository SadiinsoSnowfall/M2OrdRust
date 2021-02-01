use super::{cluster::Cluster, job::Job};

pub trait Scheduler {
	fn name(&self) -> &'static str;
	fn schedule(&self, clock: u64, jobs: &Vec<Job>, cluster: &Cluster) -> Option<usize>;
}

pub struct FCFS;

impl Scheduler for FCFS {
	fn name(&self) -> &'static str {
		"FCFS"
	}

    fn schedule(&self, _clock: u64, jobs: &Vec<Job>, cluster: &Cluster) -> Option<usize> {
		let first = jobs.first().unwrap();
		
		if cluster.available_nodes >= first.nodes {
			Some(0)
		} else {
			None
		}
    }
}

pub struct FF;

impl Scheduler for FF {
	fn name(&self) -> &'static str {
		"FF"
	}

    fn schedule(&self, _clock: u64, jobs: &Vec<Job>, cluster: &Cluster) -> Option<usize> {
		for (idx, job) in jobs.iter().enumerate() {
			if cluster.available_nodes >= job.nodes {
				return Some(idx);
			}
		}

		None
    }
}

pub struct SJF;

impl Scheduler for SJF {
	fn name(&self) -> &'static str {
		"SJF"
	}

    fn schedule(&self, _clock: u64, jobs: &Vec<Job>, cluster: &Cluster) -> Option<usize> {
		let mut min: Option<usize> = None;
		let mut min_time = None;

		for (idx, job) in jobs.iter().enumerate() {
			if cluster.available_nodes >= job.nodes {
				let jtime = job.requested_run_time;

				if min_time == None || jtime < min_time.unwrap() || (jtime == min_time.unwrap() && job.id < jobs[min.unwrap()].id) {
					min = Some(idx);
					min_time = Some(job.requested_run_time);
				}
			}
		}

		min
    }
}

pub struct FCFSEasy;

impl Scheduler for FCFSEasy {
	fn name(&self) -> &'static str {
		"FCFSEasy"
	}

    fn schedule(&self, clock: u64, jobs: &Vec<Job>, cluster: &Cluster) -> Option<usize> {
        let first = jobs.first().unwrap();
		
		if cluster.available_nodes >= first.nodes {
			Some(0)
		} else {
			// sort the running jobs by their expected end to make it easier
			let mut running = cluster.running_jobs.values().collect::<Vec<_>>();
			running.sort_unstable_by_key(|job| job.requested_run_time);

			let mut available = cluster.available_nodes;

			let mut time_before_launch = 0;
			for job in running {
				available += job.nodes;
				if available >= first.nodes {
					time_before_launch = job.expected_end - clock;
					break;
				}
			}

			for (idx, job) in jobs.iter().skip(1).enumerate() {
				if job.requested_run_time < time_before_launch && cluster.available_nodes >= job.nodes {
					return Some(idx + 1);
				}
			}

			None
		}
    }
}
