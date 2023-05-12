use std::{
    fs,
    io::{Error, BufRead},
    env,
    path::PathBuf,
    process::exit,
};
use clap::Parser;
use aitch::eprintln_help;

#[derive(Parser)]
#[command(version, about, long_about = "Print the number of slots and number of jobs.\n\nA detailed tutorial and the source code is at https://github.com/JaneliaSciComp/aitch\n\nSee also hjobs, hkill, hnslots, hstart, hstop, and hsubmit.")]

struct Args {
    /// The name of the scheduler, in the case more than one is running.  Default is all.
    #[arg(short, long)]
    name: Option<Vec<String>>,
}

fn main() {
    let args = Args::parse();

    let tmpdir = env::temp_dir();
    let mut path = PathBuf::from(&tmpdir);
    path.push("aitch");

    let schedulers = match args.name {
        Some(name) => Vec::from(name),
        None => {
            match fs::read_dir(&path) {
                Ok(content) =>
                    content.map(|res| res.map(|e| e.path().display().to_string()))
                           .collect::<Result<Vec<_>, Error>>().unwrap(),
                Err(_error) => {
                    eprintln_help(&mut path);
                    exit(1) }
            }
        }
    };

    for scheduler in schedulers.iter() {
        path.push(scheduler);

        let mut file = aitch::lock_state(&mut path);
        let nslots_total = aitch::get_nslots_total(&mut path);
        let nslots_free = aitch::get_nslots_free(&mut path);
        let reader = aitch::job_stack_reader(&mut path);
        path.pop();

        let nslots_used: Vec<usize> = nslots_total.clone().into_iter()
                                                .zip(nslots_free.clone())
                                                .map(|(x, y)| x-y).collect();

        let mut pid;
        let mut total = 0;
        let mut pending = 0;
        let mut running = 0;
        for (i,line) in reader.lines().enumerate() {
            match i % 9 {
                8 => {
                    if i==8 { continue; }
                    pid = line.unwrap();
                    total += 1;
                    if pid == "" {
                        pending += 1; }
                    if pid != "" {
                        running += 1; }
                }
                _ => {}
            }
        }

        println!("{}  {} {} {}  {} {} {}",
                 PathBuf::from(scheduler).file_name().unwrap().to_str().unwrap(),
                 nslots_total.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(","),
                 nslots_free.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(","),
                 nslots_used.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(","),
                 total, running, pending);

        file.unlock().unwrap();
    }

    if schedulers.len()>0 {
        println!("name nslots:total,free,used njobs:total,running,pending");
    }

    exit(0);
}
