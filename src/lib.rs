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

pub fn get_slot_availability(path: &mut PathBuf) -> Vec<Vec<bool>> {
    path.push("slot_availability");
    let mut slot_availability: Vec<Vec<bool>> = match fs::read_to_string(&path) {
        Ok(slot_availability_str) => {
            slot_availability_str.split("\n")
                                 .map(|s| {
                                      let mut inner_vec = Vec::new();
                                      for c in s.chars() {
                                          inner_vec.push(match c {
                                              '0' => false,
                                              '1' => true,
                                              _ => {
                                                  eprintln_help(path);
                                                  exit(1)
                                              }
                                          })
                                      }
                                      inner_vec
                                 }).collect()
        }
        Err(_error) => {
            eprintln_help(path);
            exit(1)
        }
    };
    path.pop();
    slot_availability.pop();
    return slot_availability;
}

pub fn get_nslots_total(path: &mut PathBuf) -> Vec<usize> {
    let slot_availability: Vec<Vec<bool>> = get_slot_availability(path);
    let nslots_total = slot_availability.iter().map(|x| x.len()).collect();
    return nslots_total;
}

pub fn get_nslots_free(path: &mut PathBuf) -> Vec<usize> { 
    let slot_availability: Vec<Vec<bool>> = get_slot_availability(path);
    let nslots_free = slot_availability.iter().map(|x| x.iter().filter(|b| !**b).count()).collect();
    return nslots_free;
}

pub fn update_slot_availability(path: &mut PathBuf, queue: &String, value: bool) {
    let mut slot_availability: Vec<Vec<bool>> = get_slot_availability(path);
    let queue_vec: Vec<Vec<usize>> = queue.split(";")
                                          .map(|s| {
                                              let mut inner_vec = Vec::new();
                                              if s.len()>0 {
                                                  for x in s.split(",") {
                                                      inner_vec.push(x.parse::<usize>().unwrap());
                                                  }
                                              }
                                              inner_vec
                                          }).collect();
    for (i,q) in queue_vec.iter().enumerate() {
        for s in q {
            slot_availability[i][*s] = value
        }
    }
    path.push("slot_availability");
    fs::write(&path, slot_availability.iter()
                     .map(|q| {
                          let mut inner_vec = Vec::new();
                          for s in q.iter() {
                              inner_vec.push(match s {
                                  false => "0",
                                  true => "1"
                              })
                          }
                          inner_vec.join("")
                     }).collect::<Vec<String>>().join("\n") + "\n").unwrap();
    path.pop();
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
        if iline % 9 == 0 && *line.unwrap() == id {
            reader_iter.next();
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
