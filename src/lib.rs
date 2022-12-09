use std::{
    path::PathBuf,
    fs,
    io::{BufReader, BufWriter, BufRead, Write},
    process::exit,
};
use fslock::LockFile;

pub fn eprintln_help(path: &mut PathBuf) {
    eprintln!("error reading {}, which means that aitch is either in a bad state or not running.  consider using `hstop -f` (if necessary) followed by `hstart`", path.display());
}

pub fn lock_state(path: &mut PathBuf) -> LockFile {
    path.push("lock");
    let mut file = match LockFile::open(path) {
        Ok(file) => file,
        Err(_error) => {
            eprintln_help(path);
            exit(1)
        }
    };
    file.lock().unwrap();
    path.pop();
    return file;
}

pub fn job_stack_reader(path: &mut PathBuf) -> BufReader<fs::File> {
    path.push("job_stack");
    let reader = match fs::File::open(&path) {
        Ok(job_stack) => BufReader::new(job_stack),
        Err(_error) => {
            eprintln_help(path);
            exit(1)
        }
    };
    path.pop();
    return reader;
}

pub fn get_nslots_total(path: &mut PathBuf) -> Vec<i32>{
    path.push("nslots_total");
    let nslots_total: Vec<i32> = match fs::read_to_string(&path) {
        Ok(nslots_total_str) => {
            nslots_total_str.split(",")
                            .map(|x| x.parse().unwrap())
                            .collect()
        }
        Err(_error) => {
            eprintln_help(path);
            exit(1)
        }
    };
    path.pop();
    return nslots_total;
}

pub fn get_nslots_free(path: &mut PathBuf) -> Vec<i32> { 
    path.push("nslots_free");
    let nslots_free: Vec<i32> = match fs::read_to_string(&path) {
        Ok(nslots_free_str) => {
            nslots_free_str.split(",")
                           .map(|x| x.parse().unwrap())
                           .collect()
        }
        Err(_error) => {
            eprintln_help(path);
            exit(1)
        }
    };
    path.pop();
    return nslots_free;
}

pub fn delete_job_from_stack(path: &mut PathBuf, id: String) {
    path.push("job_stack");
    let job_stack = fs::File::open(&path).unwrap();
    let reader = BufReader::new(job_stack);
    path.pop();
    path.push("job_stack_new");
    let job_stack_new = fs::File::create(&path).unwrap();
    let mut writer = BufWriter::new(job_stack_new);
    path.pop();

    let mut reader_iter = reader.lines().map(|l| l.unwrap());
    let mut iline: i32 = 0;
    loop {
        let binding = reader_iter.next();
        let line = binding.as_ref();
        if line.is_none() { break; }
        if iline % 8 == 0 && *line.unwrap() == id {
            reader_iter.next();
            reader_iter.next();
            reader_iter.next();
            reader_iter.next();
            reader_iter.next();
            reader_iter.next();
            reader_iter.next();
        } else {
            writeln!(writer, "{}", line.unwrap()).unwrap();
            iline += 1;
        }
    }
    writer.flush().ok();

    path.push("job_stack");
    fs::remove_file(&path).ok();
    let mut path_new = path.clone();
    path_new.pop();
    path_new.push("job_stack_new");
    fs::rename(&path_new, &path).ok();
    path.pop();
}

pub fn update_nslots_free(path: &mut PathBuf, nslots_required: Vec<i32>) {
    path.push("nslots_free");
    let binding = fs::read_to_string(&path).unwrap();
    let nslots_free = binding.split(",").map(|x| x.parse::<i32>().unwrap());
    let nslots_free_now = nslots_free.zip(nslots_required)
                                     .map(|(x, y)| (x+y));
    fs::write(&path, &nslots_free_now.map(|x| x.to_string()).collect::<Vec<_>>()
                                     .join(",")).ok();
    path.pop();
}
