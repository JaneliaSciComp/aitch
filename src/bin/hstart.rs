use std::{
    env,
    fs,
    path::PathBuf,
    process::exit,
    io::{BufWriter, Write},
};
use clap::Parser;
use sysinfo::{System, SystemExt, CpuRefreshKind};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// An optional name to give the scheduler, in the case more than one is needed.
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
    let nslots  = match arg {
        Some(content) => { content },
        None => { &ncpus }
    };
    let nslots_vec = nslots.split(",").map(|x| x.parse::<usize>().unwrap());

    let tmpdir = env::temp_dir();
    let mut path = PathBuf::from(&tmpdir);
    path.push("aitch");
    path.push(&args.name);

    if path.is_dir() {
        let nslots_already = aitch::get_nslots_total(&mut path);
        let nslots_equal = nslots_vec.zip(nslots_already).all(|(x, y)| x==y);
        if nslots_equal {
            eprintln!("successfully read {}, which means that aitch is already running, and the number of slots is the same", path.display());
        } else {
            eprintln!("successfully read {}, which means that aitch is already running, but the number of slots is NOT the same.  consider using `hstop` followed by `hstart`", path.display());
        }
        exit(1)
    }

    std::fs::create_dir_all(&path).unwrap();

    path.push("slot_availability");
    let slot_availability_new = fs::File::create(&path).unwrap();
    let mut writer = BufWriter::new(slot_availability_new);
    for n in nslots_vec {
        writeln!(writer, "{}", "0".repeat(n)).unwrap();
    }
    writer.flush().ok();
    path.pop();

    path.push("job_stack");
    fs::write(&path, "id\nnslots\ncommand\nvar\nout\nerr\ndep\nqueue\npid\n").unwrap();
    path.pop();

    path.push("last_jobid");
    fs::write(&path, "0").unwrap();
    path.pop();

    println!("started {} scheduler with nslots = {}", args.name, nslots);

    exit(0);
}
