use std::{
    env,
    fs,
    path::PathBuf,
    process::exit,
};
use clap::Parser;
use sysinfo::{System, SystemExt, CpuRefreshKind};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// An optional name to give the scheduler, in the case more than one are needed.
    #[arg(short, long, default_value = "default")]
    name: String,
    /// A comma-separated list of numbers denoting the total slots in each queue.  Default is one queue with as many slots as CPU cores.
    nslots: Option<String>,
}

fn main() {
    let mut sys = System::new();
    sys.refresh_cpu_specifics(CpuRefreshKind::everything());
    let ncpus = sys.cpus().len().to_string();

    let args = Args::parse();
    let arg = args.nslots.as_deref();
    let nslots_total = match arg {
        Some(content) => { content },
        None => { &ncpus }
    };

    let tmpdir = env::temp_dir();
    let mut path = PathBuf::from(&tmpdir);
    path.push("aitch");
    path.push(&args.name);

    if path.is_dir() {
        let nslots_total_already = aitch::get_nslots_total(&mut path);
        let nslots_the_same = nslots_total.split(",").map(|x| x.parse::<i32>().unwrap())
                                          .zip(nslots_total_already)
                                          .all(|(x, y)| x==y);
        if nslots_the_same {
            eprintln!("successfully read {}, which means that aitch is already running.  and the number of slots is the same.  consider using `hjobs`", path.display());
        } else {
            eprintln!("successfully read {}, which means that aitch is already running.  but the number of slots is NOT the same.  consider using `hstop` followed by `hstart`", path.display());
        }
        exit(1)
    }

    std::fs::create_dir_all(&path).unwrap();

    path.push("nslots_total");
    fs::write(&path, &nslots_total).unwrap();
    path.pop();

    path.push("nslots_free");
    fs::write(&path, &nslots_total).unwrap();
    path.pop();

    path.push("job_stack");
    fs::write(&path, "id\nnslots\ncommand\nvar\nout\nerr\ndep\npid\n").unwrap();
    path.pop();

    path.push("last_jobid");
    fs::write(&path, "0").unwrap();
    path.pop();

    println!("started {} scheduler with nslots_total = {}", args.name, nslots_total);

    exit(0);
}
