use std::{cmp::Ordering, fmt::Display, fs::File, io::{self, BufRead, BufReader}, num::ParseIntError, time::SystemTime};

use algos::Scheduler;
use min_max_heap::MinMaxHeap;

use self::{cluster::Cluster, job::Job};

pub mod job;
pub mod algos;
pub mod cluster;

#[derive(Debug, Eq, PartialEq, PartialOrd, Copy, Clone)]
#[repr(u8)]
pub enum DebugLevel {
	None,
	Info,
	Verbose
}

impl Ord for DebugLevel {
    fn cmp(&self, other: &Self) -> Ordering {
        (*self as u8).cmp(&(*other as u8))
    }
}

// the clock type used for all time measurement
type Clock = u64;

#[derive(Debug, Eq, PartialEq, PartialOrd)]
pub enum Event {
	NewJob(Job),
	JobFinished(u32),
}

impl Ord for Event {
    fn cmp(&self, other: &Self) -> Ordering {
		match (self, other) {
			(Event::NewJob(_), Event::JobFinished(_)) => Ordering::Greater,
		    (Event::JobFinished(_), Event::NewJob(_)) => Ordering::Less,
			(Event::NewJob(a), Event::NewJob(b)) => a.cmp(b),
			(Event::JobFinished(a), Event::JobFinished(b)) => a.cmp(b),
		}
    }
}

pub struct Engine {
	debug: DebugLevel,
	scheduler: Box<dyn Scheduler>,
	cluster: Cluster,
	events: MinMaxHeap<(Clock, Event)>,
	clock: Clock,
}

#[derive(Debug)]
pub enum EngineError {
	ReadError(io::Error),
	ParseError(ParseIntError)
}

impl Display for EngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
			EngineError::ReadError(why) => write!(f, "Unable to read the input file {}", why),
			EngineError::ParseError(why) => write!(f, "Unable to read the input file {}", why),
		}
    }
}

impl From<io::Error> for EngineError {
    fn from(why: io::Error) -> Self {
        EngineError::ReadError(why)
    }
}

impl From<ParseIntError> for EngineError {
	fn from(why: ParseIntError) -> Self {
        EngineError::ParseError(why)
    }
}

#[derive(Debug)]
pub struct EngineReport {
	scheduler_name: &'static str,

	makespan: Clock,
	total_completion_time: Clock,
	min_wait: Clock,
	max_wait: Clock,
	avg_wait: f64,
	median_wait: Clock,
	total_wait: Clock,

	used_ressources: u64,
	idle: u64,
	idle_percent: f64,

	time_took: u128,
}

impl Engine {
	pub fn new(scheduler: Box<dyn Scheduler>, available_nodes: u32, task_limit: Option<usize>, input_file: &str, debug: DebugLevel) -> Result<Self, EngineError> {
		if debug >= DebugLevel::Verbose {
			println!("Created a new Engine with scheduler {}", scheduler.name());
			println!("Created a new Cluster with {} nodes", available_nodes);
		}

		let mut events = MinMaxHeap::new();

		let file = File::open(input_file)?;
		let reader = BufReader::new(file);

		for line in reader.lines() {
			let line = line?;

			if line.is_empty() || line.as_bytes()[0] == b';' {
				continue;
			}

			let split = line.split_whitespace().collect::<Vec<_>>();
			assert_eq!(split.len(), 18);

			let job_id 			= split[0].parse()?;
			let submission 		= split[1].parse()?;
			let run 			= split[3].parse()?;
			let nproc: u32		= split[7].parse()?;
			let required_run 	= split[8].parse()?;
			
			let nodes = (nproc as f32 / 4.0).ceil() as u32;
			assert!(nodes > 0);


			if nodes > available_nodes {
				if debug >= DebugLevel::Verbose {
					println!("Skipping job {} as it requires {} > {} nodes", job_id, nodes, available_nodes);
				}

				continue;
			}

			// Reverse because BinaryHeap is a max-heap in Rust
			events.push((submission, Event::NewJob(Job::new(job_id, nodes, submission, run, required_run))));

			if let Some(limit) = task_limit {
				if events.len() >= limit {
					break;
				}
			}
		}

		if debug >= DebugLevel::Info {
			println!("Finished reading the input file, {} jobs will be scheduled on {} nodes. Ready for simulation", events.len(), available_nodes);
		}

		Ok(Self {
			scheduler,
			debug,
			cluster: Cluster::new(available_nodes),
			events,
			clock: 0,
		})
	}

	pub fn run(&mut self) -> EngineReport {
		if self.debug >= DebugLevel::Info {
			println!("Starting the simulation.");
		}

		let start_time = SystemTime::now();

		let mut queue = Vec::new();
		let mut wait_times = Vec::new();
		let mut completion_times = Vec::new();

		let mut scheduled_jobs = 0u32;

		while !self.events.is_empty() || !queue.is_empty() {
			if !queue.is_empty() {
				if self.debug >= DebugLevel::Verbose {
					println!("DEBUG: Jobs in the queue to schedule {:?}", queue);
				}

				while !queue.is_empty() {
					let index = match self.scheduler.schedule(self.clock, &queue, &self.cluster) {
						Some(index) => index,
						None => break
					};

					let job = queue.swap_remove(index);

					let end_time = self.clock + job.run_time;
					wait_times.push(job.wait_time_from(self.clock));
					completion_times.push(end_time);

					// Reverse because BinaryHeap is a max-heap in Rust
					self.events.push((end_time, Event::JobFinished(job.id)));
					self.cluster.schedule_job(job, self.clock);

					scheduled_jobs += 1;
					if self.debug >= DebugLevel::Info && scheduled_jobs % 1000 == 0 {
						println!("Scheduled the {}th job.", scheduled_jobs);
					}
				}
			}

			let (new_clock, event) = self.events.pop_min().unwrap(); // we already checked that the queue is not empty
			// assert!(new_clock >= self.clock);
			self.clock = new_clock;

			match event {
			    Event::NewJob(job) => {
					if self.debug >= DebugLevel::Info {
						println!("\
							DEBUG: time moved to timestamp {}. \
							Job {} was submitted now. \
							The queue now has {} jobs. \
						", self.clock, job.id, queue.len() + 1);
					}

					queue.push(job); 
				}
			    Event::JobFinished(id) => {
					self.cluster.finish_job(id);

					if self.debug >= DebugLevel::Info {
						println!("\
							DEBUG: time moved to timestamp {}. \
							Job {} finished now. \
							The cluster now has {} nodes available. \
						", self.clock, id, self.cluster.available_nodes);
					}
				}
			}
		}

		// making sure we emptied the queue too when we finished all events
		assert!(queue.is_empty());

		wait_times.sort_unstable();

		let total_wait 	= wait_times.iter().sum();
		let avg_wait 	= total_wait as f64 / wait_times.len() as f64;
		let median_wait = wait_times[wait_times.len() / 2];
		let min_wait 	= *wait_times.first().unwrap();
		let max_wait 	= *wait_times.last().unwrap();

		let total_res = self.clock * self.cluster.total_nodes as u64;
		println!("{} {} {} {}", self.clock, self.cluster.total_nodes, total_res, self.cluster.used_resources);
		let idle = total_res - self.cluster.used_resources as u64;


		EngineReport {
			scheduler_name: self.scheduler.name(),

			makespan: self.clock,
			total_completion_time: completion_times.iter().sum(),
			min_wait,
			max_wait,
			avg_wait,
			median_wait,
			total_wait,

			used_ressources: self.cluster.used_resources,
			idle,
			idle_percent: idle as f64 * 100f64 / total_res as f64,

			time_took: start_time.elapsed().unwrap().as_millis()
		}
	}
}
