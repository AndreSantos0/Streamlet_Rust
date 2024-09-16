mod domain;

use csv::ReaderBuilder;
use std::env;
use std::error::Error;
use std::fs::File;
use serde::de::DeserializeOwned;
use crate::domain::environment::Environment;
use crate::domain::node::Node;
use chrono::{Local, Timelike};
use tokio::time::{sleep, Duration};

fn my_node_start() {
    println!("Task executed at {}", Local::now().format("%H:%M:%S"));
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    match get_environment(args) {
        Ok(env) => {
            println!("Successfully read environment: {:?}", env);
        },
        Err(err) => {
            eprintln!("Error: {}", err);
            return;
        }
    }

    let now = Local::now();
    let mut next_hour = now.hour();
    let mut minute = now.minute() + 1;

    if minute >= 60 {
        next_hour += 1;
        minute -= 60;
    }

    println!("Starting at {}:{}", next_hour, minute);

    schedule_at_specific_time(next_hour, minute, my_node_start).await;
}

fn get_environment(args: Vec<String>) -> Result<Environment, Box<dyn Error>> {
    if args.len() < 3 {
        return Err("Specify Node Id and nodes´ CSV file [Streamlet_Rust.exe 1 nodes.csv]".into());
    }

    let my_id = args[1].parse::<i32>()?;
    let file_path = &args[2];
    let nodes = read_nodes_from_csv(file_path)?;

    let my_node = nodes.iter().find(|node| node.id == my_id)
        .ok_or("This process node was not found")?
        .clone();

    Ok(Environment { my_node, nodes })
}

fn read_nodes_from_csv(file_path: &str) -> Result<Vec<Node>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(file);

    let mut nodes = Vec::new();
    for result in rdr.deserialize() {
        match result {
            Ok(node) => nodes.push(node),
            Err(e) => eprintln!("Error parsing CSV line: {}", e),
        }
    }
    Ok(nodes)
}

async fn schedule_at_specific_time(hour: u32, minute: u32, task: impl Fn() -> ()) {
    let now = Local::now();
    let current_hour = now.hour();
    let current_minute = now.minute();
    let current_second = now.second();

    let target_hour = hour;
    let target_minute = minute;

    let hours_left = target_hour as u64 - current_hour as u64;
    let minutes_left = target_minute as u64 - current_minute as u64;
    let total_seconds_left = hours_left * 3600 + minutes_left * 60 - current_second as u64;

    let duration = Duration::from_secs(total_seconds_left);
    sleep(duration).await;
    task();
}