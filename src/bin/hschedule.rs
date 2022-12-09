use std::{
    env,
    fs,
    collections::{HashMap, HashSet},
    path::PathBuf,
    io::{BufRead, BufWriter, Write},
    process::{Command, exit, Stdio},
};
use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    name: String,
}

fn main() {
    let args = Args::parse();

    let tmpdir = env::temp_dir();
    let mut path = PathBuf::from(&tmpdir);
    path.push("aitch");
    path.push(&args.name);

    let mut file = aitch::lock_state(&mut path);
    let nslots_free = aitch::get_nslots_free(&mut path);
    let nslots_total = aitch::get_nslots_total(&mut path);
    let reader = aitch::job_stack_reader(&mut path);

    let mut id: String = "".to_string();
    let mut nslots: String = "".to_string();
    let mut command: String = "".to_string();
    let mut var: String = "".to_string();
    let mut out: String = "".to_string();
    let mut err: String = "".to_string();
    let mut dep = HashSet::new();
    let mut pid;
    let mut lines = Vec::new();
    let mut foundone: bool = false;
    let mut nslots_required = Vec::new();
    let mut nslots_in_excess = Vec::new();

    // scan stack for a job which fits in the free slots
    let mut prior_jobs = HashSet::new();
    for (i,line) in reader.lines().enumerate() {
        lines.push(line.as_ref().unwrap().clone());
        //lines.push(line.unwrap().clone());
        if !foundone {
            match i % 8 {
                0 => { id = line.unwrap(); },
                1 => { nslots = line.unwrap(); },
                2 => { command = line.unwrap(); },
                3 => { var = line.unwrap(); },
                4 => { out = line.unwrap(); },
                5 => { err = line.unwrap(); },
                6 => { dep = line.unwrap().split(" ").map(|x| x.to_string()).collect(); },
                7 => {
                    if i==7 { continue; }
                    prior_jobs.insert(id.clone());
                    if prior_jobs.intersection(&dep).collect::<Vec<&String>>().len() > 0 {
                        continue;
                    }
                    pid = line.unwrap();
                    nslots_required = nslots.clone()
                                            .split(",")
                                            .map(|x| x.parse::<i32>().unwrap()).collect();
                    nslots_in_excess = nslots_free.clone().into_iter()
                                                  .zip(nslots_required.clone())
                                                  .map(|(x, y)| x-y).collect();
                    if pid=="" && nslots_in_excess.clone().into_iter().all(|x| x>=0) {
                        foundone = true
                    }
                }
                _ => {}
            }
        }
    }

    // launch such a job if found
    if foundone {
        // construct command
        let mut args2: Vec<_> = shell_words::split(&command).unwrap();
        let exe = args2.remove(0);
        let mut cmd = Command::new(exe);

        let mut env_vars = HashMap::new();

        // set QUEUE environment variables
        let queue = String::from("QUEUE");
        let r: Vec<i32> = nslots_required.clone();
        let t: Vec<i32> = nslots_total;
        for (islot,e) in nslots_in_excess.clone().into_iter().enumerate() {
            let mut q = String::new();
            q += &queue;
            q += &islot.to_string();
            let v: Vec<i32> = (t[islot]-e-r[islot]..t[islot]-e).collect();
            env_vars.insert(q, v.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(","));
        }

        // set user-supplied environment variables
        if var != "" {
            let varvals: Vec<_> = var.split(" ").collect();
            for varval in varvals {
                let thisvarval: Vec<_> = varval.split("=").collect();
                env_vars.insert(thisvarval[0].to_string(), thisvarval[1].to_string());
            }
        }

        cmd.args(args2).envs(&env_vars);

        // redirection
        if out != "" && out == err {
            let outputs = fs::File::create(out).unwrap();
            let errors = outputs.try_clone().unwrap();
            cmd.stdout(Stdio::from(outputs))
               .stderr(Stdio::from(errors));
        } else {
            if out != "" {
                let outputs = fs::File::create(out).unwrap();
                cmd.stdout(outputs);
            }
            if err != "" {
                let errors = fs::File::create(err).unwrap();
                cmd.stderr(errors);
            }
        }
     
        // spawn job
        match cmd.spawn() {
            Ok(mut proc) => {
                // update nslots_free
                path.push("nslots_free");
                fs::write(&path, &nslots_in_excess.clone().into_iter()
                                                  .map(|x| x.to_string()).collect::<Vec<_>>()
                                                  .join(",")).unwrap();
                path.pop();

                // update job_stack with PID
                path.push("job_stack");
                let job_stack = fs::File::create(&path).unwrap();
                path.pop();
                let mut writer = BufWriter::new(job_stack);
                let mut lines_iter = lines.iter();
                let mut line;
                let mut iline: i32 = 0;
                loop {
                    line = lines_iter.next();
                    if line.is_none() { break; }
                    if iline % 8 == 0 && *line.unwrap() == id {
                        writeln!(writer, "{}", line.unwrap()).unwrap();
                        writeln!(writer, "{}", lines_iter.next().unwrap()).unwrap();
                        writeln!(writer, "{}", lines_iter.next().unwrap()).unwrap();
                        writeln!(writer, "{}", lines_iter.next().unwrap()).unwrap();
                        writeln!(writer, "{}", lines_iter.next().unwrap()).unwrap();
                        writeln!(writer, "{}", lines_iter.next().unwrap()).unwrap();
                        writeln!(writer, "{}", lines_iter.next().unwrap()).unwrap();
                        lines_iter.next();
                        writeln!(writer, "{}", proc.id().to_string()).unwrap();
                        iline += 8;
                    } else {
                        writeln!(writer, "{}", line.unwrap()).unwrap();
                        iline += 1;
                    }
                }
                writer.flush().unwrap();

                // wait for job to finish
                file.unlock().unwrap();
                proc.wait().unwrap();
                file.lock().unwrap();

                // update nslots_free
                aitch::update_nslots_free(&mut path, nslots_required);

                // run scheduler
                Command::new("hschedule").arg(&args.name).spawn().unwrap();
            }
            Err(error) => {
                eprintln!("error launching job {}: {}", id, error);
            }
        }

        // delete job from stack
        aitch::delete_job_from_stack(&mut path, id);

        // run scheduler
        Command::new("hschedule").arg(&args.name).spawn().unwrap();
    }

    file.unlock().unwrap();
    exit(0);
}
