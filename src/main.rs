use tokio::process::Command;
use tokio::io::{BufReader, AsyncBufReadExt};
use tokio::sync::mpsc;
use tokio::task;
use tokio::runtime::Handle;
use tokio::time::{timeout, Duration};
use std::process::Stdio;
use std::{thread, time};
use std::sync::atomic::{AtomicBool, Ordering};
use structopt::StructOpt;
use regex::Regex;

use ctf_pwn_swarmer::opts::Args;

static FLAG_GOT: AtomicBool = AtomicBool::new(false);
static VERBOSE: AtomicBool = AtomicBool::new(false);

#[tokio::main]
async fn task_spawner(tx: mpsc::Sender<String>, script: String, num_processes: u8, process_timeout: u64) {
    let mut task_counter = 0;
    loop {
        if FLAG_GOT.load(Ordering::Relaxed) {
            break;
        }
        let metrics = Handle::current().metrics();
        let active_tasks = metrics.active_tasks_count() / 2; // We spawn 2 tokio tasks for each
                                                             // run, 1 for the process and the
                                                             // other for the reader so we can
                                                             // utilize the tokio timeout func
        for _ in active_tasks..num_processes as usize {
            let tx_clone = tx.clone();
            let script_clone = script.clone();
            let curr_task_counter = task_counter + 1;
            task::spawn(async move {
                let mut child = Command::new(&script_clone)
                    .arg("SWARM")
                    .stdout(Stdio::piped())
                    .stderr(Stdio::null())
                    .spawn()
                    .expect("Failed to spawn child process");

                let stdout = child.stdout.take().expect("Failed to open stdout");
                let mut reader = BufReader::new(stdout).lines();
                task::spawn(async move {
                    while let Some(line) = reader.next_line().await.unwrap() {
                        tx_clone.send(line).await.expect("Failed to send line");
                    }
                });

                if let Err(_) = timeout(Duration::from_secs(process_timeout), child.wait()).await {
                    println!("Timeout on task {}", curr_task_counter);
                    child.kill().await.expect("Failed to kill process");
                }
            });
            task_counter += 1;
            println!("Spawned task {}", task_counter);
        }
        thread::sleep(time::Duration::from_secs(1));
    }
}

#[tokio::main]
async fn main() {
    let args = Args::from_args();

    let script = args.script;
    let num_processes = args.num_processes;
    let process_timeout = args.timeout;
    let verbose = args.verbose;
    let flag_format = &args.flag_format.unwrap_or_else(|| "\0\0".to_string());
    let flag_format_regex = &args.flag_format_regex.unwrap_or_else(|| "\0\0".to_string());
    let re = Regex::new(flag_format_regex).expect("Invalid Regex");
    
    println!("num_processes: {:?}, flag_format: {:?}, flag_format_regex: {:?}", args.num_processes, flag_format, flag_format_regex);

    let (tx, mut rx) = mpsc::channel::<String>(100);
    
    VERBOSE.store(verbose, Ordering::Relaxed);
    thread::spawn(move || {
        task_spawner(tx, script, num_processes, process_timeout);
    });
 
    while let Some(line) = rx.recv().await {
        if VERBOSE.load(Ordering::Relaxed) {
            println!("[VERBOSE] {}", line);
        }
        if flag_format != "\0\0" && line.contains(flag_format) {
            FLAG_GOT.store(true, Ordering::Relaxed);
            let flag_ind_start = line.find(flag_format).unwrap();
            let flag_ind_end = line.find("}").unwrap();
            println!("{}", &line[flag_ind_start..flag_ind_end+1]);
            break;
        }
        if flag_format_regex != "\0\0" && re.is_match(&line) {
            FLAG_GOT.store(true, Ordering::Relaxed);
            let flag = re.find(&line).unwrap();
            println!("{}", flag.as_str()); 
            break;
        }

    }
}

