use std::time::SystemTime;

use algos::{Scheduler};
use sim::{DebugLevel, Engine, algos};

pub mod sim;

fn main() {
    let data_file = "ANL-Intrepid-2009-1.swf";


    let engines: &[Box<dyn Scheduler>] = &[Box::new(algos::FCFS), Box::new(algos::FF), Box::new(algos::SJF), Box::new(algos::FCFSEasy)];
    let node_counts = &[64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384, 32768, 65536, 131072];
    
    let start_time = SystemTime::now();

    for _ in node_counts {
        for _ in engines {
            let mut engine = match Engine::new(Box::new(algos::FCFS), 1024, None, data_file, DebugLevel::None) {
                Ok(engine) => engine,
                Err(why) => panic!("Error during engine initialization: {}", why)
            };

            let report = engine.run();
            println!("{:?}", report);
        }
    }

    let ellapsed = start_time.elapsed().unwrap();
    println!("\n\ntook {}", humantime::format_duration(ellapsed));

}
