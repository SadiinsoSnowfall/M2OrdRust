

use hashbrown::HashMap;

use super::{Clock, job::Job};

pub struct Cluster {
	pub total_nodes: u32,
	pub available_nodes: u32,
	pub used_resources: u64,
	pub running_jobs: HashMap<u32, Job>,
}

impl Cluster {
	pub fn new(nodes: u32) -> Self {
		Self {
			total_nodes: nodes,
			available_nodes: nodes,
			used_resources: 0,
			running_jobs: HashMap::new()
		}
	}

	pub fn schedule_job(&mut self, job: Job, clock: Clock) -> bool {
		if job.nodes > self.available_nodes {
			println!("[{}] Job {} is trying to run on {} but only {} are available.", clock, job.id, job.nodes, self.available_nodes);
			return false;
		}

		self.available_nodes -= job.nodes;
		let mut job = job;
		job.set_scheduled(clock);
		
		self.running_jobs.insert(job.id, job);
		true
	}

	pub fn finish_job(&mut self, job_id: u32) {
		if let Some(job) = self.running_jobs.remove(&job_id) {
			self.available_nodes += job.nodes;
			self.used_resources += job.nodes as u64 * job.run_time;
		}
	}

	pub fn print_stats(&self, makespan: Clock) {
		let total_resources = makespan * self.total_nodes as u64;
		let idle = total_resources - self.used_resources;
		let idle_percent = idle * 100 / total_resources;
		
		println!("\
			Usage of the  machine:\n\
			- {} node-seconds used\n\
			- from {} available\n\
			- Nodes spent {} seconds in idle,\
			\tor {}%.
		", self.used_resources, total_resources, idle, idle_percent);
	}
}
