use std::{
    env,
    path::PathBuf,
    process::exit,
};
use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The name of the scheduler, in the case more than one is running.
    #[arg(short, long, default_value = "default")]
    name: String,
    /// Optionally output just the "used" or "free" jobs
    kind: Option<String>,
}

fn main() {
    let args = Args::parse();

    let tmpdir = env::temp_dir();
    let mut path = PathBuf::from(&tmpdir);
    path.push("aitch");
    path.push(&args.name);

    let mut file = aitch::lock_state(&mut path);
    let nslots_total = aitch::get_nslots_total(&mut path);
    let nslots_free = aitch::get_nslots_free(&mut path);

    match args.kind.as_deref() {
        None => println!("{}",
                          nslots_total.iter()
                                      .map(|x| x.to_string())
                                      .collect::<Vec<_>>()
                                      .join(",")),
        Some("used") => println!("{}",
                         nslots_total.iter()
                                     .zip(nslots_free.iter())
                                     .map(|(x, y)| (x-y).to_string())
                                     .collect::<Vec<_>>()
                                     .join(",")),
        Some("free") => println!("{}",
                         nslots_free.iter()
                                    .map(|x| x.to_string())
                                    .collect::<Vec<_>>()
                                    .join(",")),
        _ => {
            eprintln!("unrecognized optional argument.  \
                       if one is supplied, it must be `used` or `free`");
            exit(1);
        }
    }

    file.unlock().unwrap();
    exit(0);
}
