use std::{
    env,
    path::PathBuf,
    io::BufRead,
    process::exit,
};
use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = "Print the details of all jobs in the queue.\n\nA detailed tutorial and the source code is at https://github.com/JaneliaSciComp/aitch\n\nSee also hkill, hnslots, hstart, hstatus, hstop, and hsubmit.")]
struct Args {
    /// The name of the scheduler, in the case more than one is running.
    #[arg(short, long, default_value = "default")]
    name: String,
    /// Optionally output just the "running" or "pending" jobs, or a single job ID
    kind: Option<String>,
}

fn main() {
    let args = Args::parse();
    let kind =  args.kind.as_deref();

    let tmpdir = env::temp_dir();
    let mut path = PathBuf::from(&tmpdir);
    path.push("aitch");
    path.push(&args.name);

    let mut file = aitch::lock_state(&mut path);
    let reader = aitch::job_stack_reader(&mut path);

    let mut id: String = "".to_string();
    let mut nslots: String = "".to_string();
    let mut command: String = "".to_string();
    let mut var: String = "".to_string();
    let mut out: String = "".to_string();
    let mut err: String = "".to_string();
    let mut append: bool = false;
    let mut dep: String = "".to_string();
    let mut queue: String = "".to_string();
    let mut pid;
    let mut printedsomething: bool = false;

    for (i,line) in reader.lines().enumerate() {
        match i % 10 {
            0 => { id = line.unwrap(); },
            1 => { nslots = line.unwrap(); },
            2 => { command = line.unwrap(); },
            3 => { var = line.unwrap(); },
            4 => { out = line.unwrap(); },
            5 => { err = line.unwrap(); },
            6 => { if i>6 { append = line.unwrap().trim().parse().unwrap(); } },
            7 => { dep = line.unwrap(); },
            8 => { queue = line.unwrap(); },
            9 => {
                if i==9 { continue; }
                pid = line.unwrap();
                match kind {
                    None => {
                        println!("{} {} {} {} {} {} {} {} {} {}",
                                 id, nslots, command, var, out, err, append, dep, queue, pid);
                        printedsomething = true;
                    }
                    Some("pending") => {
                        if pid == "" {
                            println!("{} {} {} {} {} {} {} {} {} {}",
                                     id, nslots, command, var, out, err, append, dep, queue, pid);
                            printedsomething = true;
                        }
                    }
                    Some("running") => {
                        if pid != "" {
                            println!("{} {} {} {} {} {} {} {} {} {}",
                                      id, nslots, command, var, out, err, append, dep, queue, pid);
                            printedsomething = true;
                        }
                    }
                    Some(_x) => {
                        if id == kind.unwrap() {
                            println!("{} {} {} {} {} {} {} {} {} {}",
                                      id, nslots, command, var, out, err, append, dep, queue, pid);
                            printedsomething = true;
                            break;
                        }
                    }
                }
            }
            _ => {}
        }
    }

    file.unlock().unwrap();

    if printedsomething {
        println!("id nslots command var out err append dep queue pid");
    } else if kind == None || kind == Some("pending") || kind == Some("running") {
        println!("no jobs found");
    } else {
        eprintln!("no such job found");
        exit(1);
    }
    exit(0);
}
