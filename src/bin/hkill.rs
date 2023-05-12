use std::{
    env,
    path::PathBuf,
    io::BufRead,
    process::{Command, exit},
};
use std::str::FromStr;
use sysinfo::Signal::*;
use sysinfo::{Pid, ProcessExt, System, SystemExt, RefreshKind, ProcessRefreshKind};
use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = "Terminate a specific job and remove it from the queue.\n\nA detailed tutorial and the source code is at https://github.com/JaneliaSciComp/aitch\n\nSee also hjobs, hnslots, hstart, hstatus, hstop, and hsubmit.")]
struct Args {
    /// The name of the scheduler, in the case more than one is running.
    #[arg(short, long, default_value = "default")]
    name: String,
    /// The identification number of the job of interest.
    jobid: String,
    /// Send SIGKILL (instead of the default SIGTERM)
    #[arg(short, long)]
    kill: bool,
    /// Whether to remove a job from aitch's stack if the PID can't be found
    #[arg(short, long)]
    force: bool,
}

fn main() {
    let args = Args::parse();

    let tmpdir = env::temp_dir();
    let mut path = PathBuf::from(&tmpdir);
    path.push("aitch");
    path.push(&args.name);

    let mut file = aitch::lock_state(&mut path);
    let reader = aitch::job_stack_reader(&mut path);

    let mut id: String = "".to_string();
    let mut queue: String = "".to_string();
    let mut foundone: bool = false;
    let r = RefreshKind::new();
    let r = r.with_processes(ProcessRefreshKind::everything());
    let sys = System::new_with_specifics(r);

    for (i,line) in reader.lines().enumerate() {
        match i % 9 {
            0 => { id = line.unwrap(); },
            7 => { queue = line.unwrap(); },
            8 => {
                if i==8 { continue; }
                let cloned_line = line.unwrap().clone();
                if cloned_line == "" { continue; }
                let pid = Pid::from_str(&cloned_line).unwrap();
                if id == args.jobid {
                    match sys.process(pid) {
                        Some(p) => {
                            if args.kill {
                                if p.kill_with(Kill).is_none() {
                                    eprintln!("SIGKILL not supported on this platform");
                                }
                            } else {
                                if p.kill_with(Term).is_none() {
                                    eprintln!("SIGTERM not supported on this platform.  use --kill to send SIGKILL instead");
                                }
                            }
                        }
                        None => {
                            if args.force {
                                aitch::update_slot_availability(&mut path, &queue, false);
                                aitch::delete_job_from_stack(&mut path, id);
                            } else {
                                eprintln!("couldn't find PID.  use --force to delete job from aitch's stack");
                            }
                        }
                    };
                    foundone = true;
                    break;
                }
            }
            _ => {}
        }
    }

    file.unlock().unwrap();
    if foundone {
        Command::new("hschedule").arg(args.name).spawn().unwrap();
        exit(0);
    } else {
        eprintln!("couldn't find job {}", args.jobid);
        exit(1);
    }
}
