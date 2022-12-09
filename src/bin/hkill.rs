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
#[command(author, version, about, long_about = None)]
struct Args {
    /// The name of the scheduler, in the case more than one are running.
    #[arg(short, long, default_value = "default")]
    name: String,
    /// The identification number of the job of interest.
    jobid: String,
    /// Whether to remove a job from the stack if the PID can't be found
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
    let mut nslots: String = "".to_string();
    let mut foundone: bool = false;
    let r = RefreshKind::new();
    let r = r.with_processes(ProcessRefreshKind::everything());
    let sys = System::new_with_specifics(r);

    for (i,line) in reader.lines().enumerate() {
        match i % 8 {
            0 => { id = line.unwrap(); },
            1 => { nslots = line.unwrap(); },
            7 => {
                if i==7 { continue; }
                let cloned_line = line.unwrap().clone();
                if cloned_line == "" { continue; }
                let pid = Pid::from_str(&cloned_line).unwrap();
                if id == args.jobid {
                    match sys.process(pid) {
                        Some(p) => {
                            if let Some(_res) = p.kill_with(Kill) {
                                //println!("kill: {res}");
                            } else {
                                eprintln!("kill: signal not supported on this platform");
                            }
                        }
                        None => {
                            if args.force {
                                let nslots_required = nslots.clone()
                                                            .split(",")
                                                            .map(|x| x.parse::<i32>().unwrap()).collect();
                                aitch::update_nslots_free(&mut path, nslots_required);
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
