use std::{
    env,
    fs,
    io::Error,
    path::PathBuf,
    io::BufRead,
    process::exit,
};
use std::str::FromStr;
use sysinfo::Signal::*;
use sysinfo::{Pid, ProcessExt, System, SystemExt};
use clap::{Parser, ArgGroup};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(group(ArgGroup::new("vers") .args(["name", "all"])))]
struct Args {
    /// The name of the scheduler, in the case more than one are running.
    #[arg(short, long, default_value = "default")]
    name: Option<Vec<String>>,
    /// Stop every scheduler.  Mutually exclusive with --name.
    #[arg(short, long)]
    all: bool,
    /// Whether to still stop if jobs are outstanding.
    #[arg(short, long)]
    force: bool,
}

fn main() {
    let args = Args::parse();

    let tmpdir = env::temp_dir();
    let mut path = PathBuf::from(&tmpdir);
    path.push("aitch");

    let schedulers = if args.all {
        fs::read_dir(&path).unwrap()
                    .map(|res| res.map(|e| e.path().display().to_string()))
                    .collect::<Result<Vec<_>, Error>>().unwrap()
    } else {
        Vec::from(args.name.unwrap())
    };

    for scheduler in schedulers.iter() {
        path.push(scheduler);

        let mut file = aitch::lock_state(&mut path);
        let reader = aitch::job_stack_reader(&mut path);

        let sys = System::new_all();

        for (i,line) in reader.lines().enumerate() {
            match i % 8 {
                7 => {
                    if i==7 { continue; }
                    let cloned_line = line.unwrap().clone();
                    if !args.force {
                        eprintln!("jobs are still queued.  use --force to stop anyway");
                        exit(1);
                    }
                    if cloned_line == "" { continue; }
                    let pid = Pid::from_str(&cloned_line).unwrap();
                    match sys.process(pid) {
                        Some(p) => {
                            if let Some(_res) = p.kill_with(Kill)
                            {
                                //println!("kill: {res}");
                            } else {
                                eprintln!("kill: signal not supported on this platform");
                            }
                        }
                        None => { }
                    };
                }
                _ => {}
            }
        }

        fs::remove_dir_all(&path).unwrap();
        path.pop();

        file.unlock().unwrap();
    }

    exit(0);
}
