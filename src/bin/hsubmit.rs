use std::{
    env,
    fs,
    path::PathBuf,
    io::{BufRead, BufReader, Write},
    process::{Command, exit},
};
use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = "Add a new job to the queue.\n\nA detailed tutorial and the source code is at https://github.com/JaneliaSciComp/aitch\n\nSee also hjobs, hkill, hnslots, hstart, hstatus, and hstop.")]
struct Args {
    /// The name of the scheduler, in the case more than one is running.
    #[arg(short, long, default_value = "default")]
    name: String,
    /// VARIABLE=VALUE.  Set VARIABLE equal to VALUE when running command.  This option can be used multiple times.
    #[arg(short, long)]
    var: Option<Vec<String>>,
    /// Path to file in which to save the standard output
    #[arg(short, long)]
    out: Option<String>,
    /// Path to file in which to save the standard error
    #[arg(short, long)]
    err: Option<String>,
    /// The identification number of a job that must finish first
    #[arg(short, long)]
    dep: Option<Vec<String>>,
    /// A comma-separated list of numbers denoting the required slots in each queue.
    #[clap(allow_hyphen_values = true)]
    nslots: String,
    /// The command to execute
    command: Vec<String>,
}

fn main() {
    let args = Args::parse();

    let tmpdir = env::temp_dir();
    let mut path = PathBuf::from(&tmpdir);
    path.push("aitch");
    path.push(&args.name);

    let mut file = aitch::lock_state(&mut path);
    let nslots_total = aitch::get_nslots_total(&mut path);

    let nslots_required = args.nslots.split(',')
                                     .map(|x| x.parse::<i32>().unwrap())
                                     .zip(nslots_total.into_iter())
                                     .map(|(x,y)| if x>=0 {x} else {y.try_into().unwrap()})
                                     .map(|x| x.to_string())
                                     .collect::<Vec<String>>()
                                     .join(",");

    let var = match args.var {
        Some(content) => { content.join(" ") },
        None => { "".to_string() }
    };

    path.push("last_jobid");
    let fid = fs::File::open(&path).unwrap();
    let mut buffer = BufReader::new(fid);
    let mut first_line = String::new();
    buffer.read_line(&mut first_line).unwrap();
    let id = (1+first_line.parse::<usize>().unwrap()).to_string();
    path.pop();

    let out = match args.out {
        Some(content) => { content },
        None => {
            path.push(id.clone()+".out");
            let tmp = path.clone().into_os_string().into_string().unwrap();
            path.pop();
            tmp
        }
    };

    let err = match args.err {
        Some(content) => { content },
        None => {
            path.push(id.clone()+".err");
            let tmp = path.clone().into_os_string().into_string().unwrap();
            path.pop();
            tmp
        }
    };

    let dep = match args.dep {
        Some(content) => { content.join(" ") },
        None => "".to_string(),
    };

    path.push("job_stack");
    let mut f = fs::OpenOptions::new().write(true).append(true).open(&path).unwrap();
    writeln!(f, "{}", id).unwrap();
    writeln!(f, "{}", nslots_required).unwrap();
    writeln!(f, "{}", args.command.join(" ")).unwrap();
    writeln!(f, "{}", var).unwrap();
    writeln!(f, "{}", out).unwrap();
    writeln!(f, "{}", err).unwrap();
    writeln!(f, "{}", dep).unwrap();
    writeln!(f, "").unwrap();
    writeln!(f, "").unwrap();
    path.pop();

    path.push("last_jobid");
    fs::write(&path, &id).unwrap();
    path.pop();

    file.unlock().unwrap();

    println!("{}", id);

    Command::new("hschedule").arg(args.name).spawn().unwrap();

    exit(0);
}
