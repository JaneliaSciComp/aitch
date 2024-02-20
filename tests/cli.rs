use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::{
    process::{Command, Stdio, Child},
    io::{BufReader, BufRead, Read},
    env,
    path::PathBuf,
    thread,
    time,
    fs::File,
};

fn wait_for_all_jobs_to_finish(mut cmd: Command) {
    let mut n = 0;
    while n<10 && cmd.stdout(Stdio::piped()).stderr(Stdio::piped()).status().unwrap().success() {
        thread::sleep(time::Duration::from_secs(1));
        n += 1;
    }
    assert_ne!(n, 10);
}

#[test]
fn not_running() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("hstop")?;
    cmd.args(["--name", "not_running"])
       .arg("--force").stderr(Stdio::piped()).status()?;

    let tmpdir = env::temp_dir();
    let mut path = PathBuf::from(&tmpdir);
    path.push("aitch");
    path.push("not_running");

    let predicate_fn = predicate::path::is_dir();
    assert_eq!(false, predicate_fn.eval(&path));

    let mut cmd = Command::cargo_bin("hjobs")?;
    cmd.args(["--name", "not_running"])
       .assert().failure().stderr(predicate::str::contains("bad state or not running"));

    let mut cmd = Command::cargo_bin("hkill")?;
    cmd.args(["--name", "not_running"])
       .arg("1");
    cmd.assert().failure().stderr(predicate::str::contains("bad state or not running"));

    let mut cmd = Command::cargo_bin("hnslots")?;
    cmd.args(["--name", "not_running"])
       .assert().failure().stderr(predicate::str::contains("bad state or not running"));

    let mut cmd = Command::cargo_bin("hstop")?;
    cmd.args(["--name", "not_running"])
       .assert().failure().stderr(predicate::str::contains("bad state or not running"));

    let mut cmd = Command::cargo_bin("hsubmit")?;
    cmd.args(["--name", "not_running"])
       .arg("1");
    if env::consts::OS == "windows" {
        cmd.args(["powershell", "--", "-command", "sleep", "5"]);
    } else {
        cmd.args(["sleep", "5"]);
    }
    cmd.assert().failure().stderr(predicate::str::contains("bad state or not running"));

    let mut cmd = Command::cargo_bin("hstart")?;
    cmd.args(["--name", "not_running"])
       .assert().success().stdout(predicate::str::contains("scheduler with nslots"));

    assert_eq!(true, predicate_fn.eval(&path));
    let predicate_fn = predicate::path::is_file();
    path.push("job_stack");           assert_eq!(true, predicate_fn.eval(&path));  path.pop();
    path.push("last_jobid");          assert_eq!(true, predicate_fn.eval(&path));  path.pop();
    path.push("slot_availability");   assert_eq!(true, predicate_fn.eval(&path));  path.pop();

    let mut cmd = Command::cargo_bin("hstop")?;
    cmd.args(["--name", "not_running"])
       .assert().success();

    Ok(())
}

fn assert_stdout(child: &mut Child, output: &str) {
    let stdout = child.stdout.take().unwrap();

    let mut bufread = BufReader::new(stdout);
    let mut buf = String::new();

    if let Ok(_n) = bufread.read_line(&mut buf) {
        assert_eq!(buf, output);
    }
}

#[test]
fn one_short_job() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("hstop")?;
    cmd.args(["--name", "one_short_job"])
       .arg("--force").stderr(Stdio::piped()).status()?;

    let mut cmd = Command::cargo_bin("hstart")?;
    cmd.args(["--name", "one_short_job"])
       .arg("1")
       .assert().success().stdout(predicate::str::contains("scheduler with nslots"));

    let mut cmd = Command::cargo_bin("hnslots")?;
    cmd.args(["--name", "one_short_job"])
       .assert().success().stdout(predicate::eq("1\n"));

    let mut cmd = Command::cargo_bin("hjobs")?;
    cmd.args(["--name", "one_short_job"])
       .assert().success().stdout(predicate::str::contains("no jobs found"));

    let tmpdir = env::temp_dir();
    let mut path = PathBuf::from(&tmpdir);
    path.push("aitch");
    path.push("one_short_job");

    path.push("shortjob");
    let predicate_fn = predicate::path::is_file();
    assert_eq!(false, predicate_fn.eval(&path));

    let mut cmd = Command::cargo_bin("hsubmit")?;
    cmd.args(["--name", "one_short_job"])
       .arg("1");
    if env::consts::OS == "windows" {
        let escaped_path = "'".to_string()+path.display().to_string().as_str()+"'";
        cmd.args(["powershell", "--", "-command", "ni", &escaped_path]);
    } else {
        cmd.args(["touch", path.display().to_string().as_str()]);
    }
    let mut child = cmd.stdout(Stdio::piped())
                       .spawn().unwrap();
    assert_stdout(&mut child, "1\n");

    thread::sleep(time::Duration::from_secs(1));
       
    let predicate_fn = predicate::path::is_file();
    assert_eq!(true, predicate_fn.eval(&path));
    path.pop();

    let mut cmd = Command::cargo_bin("hjobs")?;
    cmd.args(["--name", "one_short_job"])
       .assert().success().stdout(predicate::str::contains("\n").count(1));

    let mut cmd = Command::cargo_bin("hstop")?;
    cmd.args(["--name", "one_short_job"])
       .assert().success();

    Ok(())
}

#[test]
fn one_long_job() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("hstop")?;
    cmd.args(["--name", "one_long_job"])
       .arg("--force").stderr(Stdio::piped()).status()?;

    let mut cmd = Command::cargo_bin("hstart")?;
    cmd.args(["--name", "one_long_job"])
       .arg("1")
       .assert().success().stdout(predicate::str::contains("scheduler with nslots"));

    let mut cmd = Command::cargo_bin("hsubmit")?;
    cmd.args(["--name", "one_long_job"])
       .arg("1");
    if env::consts::OS == "windows" {
        cmd.args(["powershell", "--", "-command", "sleep", "5"]);
    } else {
        cmd.args(["sleep", "5"]);
    }
    let mut child = cmd.stdout(Stdio::piped())
                       .spawn().unwrap();
    assert_stdout(&mut child, "1\n");

    thread::sleep(time::Duration::from_secs(1));
       
    let mut cmd = Command::cargo_bin("hjobs")?;
    cmd.args(["--name", "one_long_job"])
       .assert().success().stdout(predicate::str::contains("\n").count(2));

    let mut cmd = Command::cargo_bin("hjobs")?;
    cmd.args(["--name", "one_long_job"])
       .arg("running")
       .assert().success().stdout(predicate::str::contains("\n").count(2));

    let mut cmd = Command::cargo_bin("hjobs")?;
    cmd.args(["--name", "one_long_job"])
       .arg("pending")
       .assert().success().stdout(predicate::str::contains("\n").count(1));

    let mut cmd = Command::cargo_bin("hjobs")?;
    cmd.args(["--name", "one_long_job"])
       .arg("1");
    cmd.assert().success().stdout(predicate::str::contains("\n").count(2));

    wait_for_all_jobs_to_finish(cmd);

    let mut cmd = Command::cargo_bin("hjobs")?;
    cmd.args(["--name", "one_long_job"])
       .assert().success().stdout(predicate::str::contains("\n").count(1));

    let mut cmd = Command::cargo_bin("hstop")?;
    cmd.args(["--name", "one_long_job"])
       .assert().success();

    Ok(())
}

#[test]
fn two_jobs() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("hstop")?;
    cmd.args(["--name", "two_jobs"])
       .arg("--force").stderr(Stdio::piped()).status()?;

    let mut cmd = Command::cargo_bin("hstart")?;
    cmd.args(["--name", "two_jobs"])
       .arg("1")
       .assert().success().stdout(predicate::str::contains("scheduler with nslots"));

    let mut cmd = Command::cargo_bin("hsubmit")?;
    cmd.args(["--name", "two_jobs"])
       .arg("1");
    if env::consts::OS == "windows" {
        cmd.args(["powershell", "--", "-command", "sleep", "5"]);
    } else {
        cmd.args(["sleep", "5"]);
    }
    let mut child = cmd.stdout(Stdio::piped())
                       .spawn().unwrap();
    assert_stdout(&mut child, "1\n");

    let tmpdir = env::temp_dir();
    let mut path = PathBuf::from(&tmpdir);
    path.push("aitch");
    path.push("two_jobs");

    path.push("secondjob");
    let predicate_fn = predicate::path::is_file();
    assert_eq!(false, predicate_fn.eval(&path));

    let mut cmd = Command::cargo_bin("hsubmit")?;
    cmd.args(["--name", "two_jobs"])
       .arg("1");
    if env::consts::OS == "windows" {
        let escaped_path = "'".to_string()+path.display().to_string().as_str()+"'";
        cmd.args(["powershell", "--", "-command", "ni", &escaped_path]);
    } else {
        cmd.args(["touch", path.display().to_string().as_str()]);
    }
    let mut child = cmd.stdout(Stdio::piped())
                       .spawn().unwrap();
    assert_stdout(&mut child, "2\n");

    thread::sleep(time::Duration::from_secs(1));
       
    let mut cmd = Command::cargo_bin("hjobs")?;
    cmd.args(["--name", "two_jobs"])
       .assert().success().stdout(predicate::str::contains("\n").count(3));

    let mut cmd = Command::cargo_bin("hjobs")?;
    cmd.args(["--name", "two_jobs"])
       .arg("running")
       .assert().success().stdout(predicate::str::contains("\n").count(2));

    let mut cmd = Command::cargo_bin("hjobs")?;
    cmd.args(["--name", "two_jobs"])
       .arg("pending")
       .assert().success().stdout(predicate::str::contains("\n").count(2));

    let mut cmd = Command::cargo_bin("hjobs")?;
    cmd.args(["--name", "two_jobs"])
       .arg("2");
    cmd.assert().success().stdout(predicate::str::contains("\n").count(2));

    wait_for_all_jobs_to_finish(cmd);

    let predicate_fn = predicate::path::is_file();
    assert_eq!(true, predicate_fn.eval(&path));
    path.pop();

    let mut cmd = Command::cargo_bin("hjobs")?;
    cmd.args(["--name", "two_jobs"])
       .assert().success().stdout(predicate::str::contains("\n").count(1));

    let mut cmd = Command::cargo_bin("hstop")?;
    cmd.args(["--name", "two_jobs"])
       .assert().success();

    Ok(())
}

#[test]
fn two_slots() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("hstop")?;
    cmd.args(["--name", "two_slots"])
       .arg("--force").stderr(Stdio::piped()).status()?;

    let mut cmd = Command::cargo_bin("hstart")?;
    cmd.args(["--name", "two_slots"])
       .arg("2")
       .assert().success().stdout(predicate::str::contains("scheduler with nslots"));

    let mut cmd = Command::cargo_bin("hsubmit")?;
    cmd.args(["--name", "two_slots"])
       .arg("1");
    if env::consts::OS == "windows" {
        cmd.args(["powershell", "--", "-command", "sleep", "5"]);
    } else {
        cmd.args(["sleep", "5"]);
    }
    let mut child = cmd.stdout(Stdio::piped())
                       .spawn().unwrap();
    assert_stdout(&mut child, "1\n");

    let tmpdir = env::temp_dir();
    let mut path = PathBuf::from(&tmpdir);
    path.push("aitch");
    path.push("two_slots");

    path.push("secondslot");
    let predicate_fn = predicate::path::is_file();
    assert_eq!(false, predicate_fn.eval(&path));

    let mut cmd = Command::cargo_bin("hsubmit")?;
    cmd.args(["--name", "two_slots"])
       .arg("1");
    if env::consts::OS == "windows" {
        let escaped_path = "'".to_string()+path.display().to_string().as_str()+"'";
        cmd.args(["powershell", "--", "-command", "ni", &escaped_path]);
    } else {
        cmd.args(["touch", path.display().to_string().as_str()]);
    }
    let mut child = cmd.stdout(Stdio::piped())
                       .spawn().unwrap();
    assert_stdout(&mut child, "2\n");

    thread::sleep(time::Duration::from_secs(1));
       
    let mut cmd = Command::cargo_bin("hjobs")?;
    cmd.args(["--name", "two_slots"])
       .assert().success().stdout(predicate::str::contains("\n").count(2));

    let mut cmd = Command::cargo_bin("hjobs")?;
    cmd.args(["--name", "two_slots"])
       .arg("running")
       .assert().success().stdout(predicate::str::contains("\n").count(2));

    let mut cmd = Command::cargo_bin("hjobs")?;
    cmd.args(["--name", "two_slots"])
       .arg("pending")
       .assert().success().stdout(predicate::str::contains("\n").count(1));

    let mut cmd = Command::cargo_bin("hjobs")?;
    cmd.args(["--name", "two_slots"])
       .arg("2");
    cmd.assert().failure().stderr(predicate::str::contains("no such job found"));

    let mut cmd = Command::cargo_bin("hjobs")?;
    cmd.args(["--name", "two_slots"])
       .arg("1");
    cmd.assert().success().stdout(predicate::str::contains("\n").count(2));

    wait_for_all_jobs_to_finish(cmd);

    let predicate_fn = predicate::path::is_file();
    assert_eq!(true, predicate_fn.eval(&path));
    path.pop();

    let mut cmd = Command::cargo_bin("hjobs")?;
    cmd.args(["--name", "two_slots"])
       .assert().success().stdout(predicate::str::contains("\n").count(1));

    let mut cmd = Command::cargo_bin("hstop")?;
    cmd.args(["--name", "two_slots"])
       .assert().success();

    Ok(())
}

#[test]
fn a_dependent_job() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("hstop")?;
    cmd.args(["--name", "a_dependent_job"])
       .arg("--force").stderr(Stdio::piped()).status()?;

    let mut cmd = Command::cargo_bin("hstart")?;
    cmd.args(["--name", "a_dependent_job"])
       .arg("2")
       .assert().success().stdout(predicate::str::contains("scheduler with nslots"));

    let mut cmd = Command::cargo_bin("hsubmit")?;
    cmd.args(["--name", "a_dependent_job"])
       .arg("1");
    if env::consts::OS == "windows" {
        cmd.args(["powershell", "--", "-command", "sleep", "5"]);
    } else {
        cmd.args(["sleep", "5"]);
    }
    let mut child = cmd.stdout(Stdio::piped())
                       .spawn().unwrap();
    assert_stdout(&mut child, "1\n");

    let tmpdir = env::temp_dir();
    let mut path = PathBuf::from(&tmpdir);
    path.push("aitch");
    path.push("a_dependent_job");

    path.push("dependentjob");
    let predicate_fn = predicate::path::is_file();
    assert_eq!(false, predicate_fn.eval(&path));

    let mut cmd = Command::cargo_bin("hsubmit")?;
    cmd.args(["--name", "a_dependent_job"])
       .arg("1")
       .args(["--dep", "1"]);
    if env::consts::OS == "windows" {
        let escaped_path = "'".to_string()+path.display().to_string().as_str()+"'";
        cmd.args(["powershell", "--", "-command", "ni", &escaped_path]);
    } else {
        cmd.args(["touch", path.display().to_string().as_str()]);
    }
    let mut child = cmd.stdout(Stdio::piped())
                       .spawn().unwrap();
    assert_stdout(&mut child, "2\n");

    thread::sleep(time::Duration::from_secs(1));
       
    let mut cmd = Command::cargo_bin("hjobs")?;
    cmd.args(["--name", "a_dependent_job"])
       .assert().success().stdout(predicate::str::contains("\n").count(3));

    let mut cmd = Command::cargo_bin("hjobs")?;
    cmd.args(["--name", "a_dependent_job"])
       .arg("running")
       .assert().success().stdout(predicate::str::contains("\n").count(2));

    let mut cmd = Command::cargo_bin("hjobs")?;
    cmd.args(["--name", "a_dependent_job"])
       .arg("pending")
       .assert().success().stdout(predicate::str::contains("\n").count(2));

    let mut cmd = Command::cargo_bin("hjobs")?;
    cmd.args(["--name", "a_dependent_job"])
       .arg("2");
    cmd.assert().success().stdout(predicate::str::contains("\n").count(2));

    wait_for_all_jobs_to_finish(cmd);

    let predicate_fn = predicate::path::is_file();
    assert_eq!(true, predicate_fn.eval(&path));
    path.pop();

    let mut cmd = Command::cargo_bin("hjobs")?;
    cmd.args(["--name", "a_dependent_job"])
       .assert().success().stdout(predicate::str::contains("\n").count(1));

    let mut cmd = Command::cargo_bin("hstop")?;
    cmd.args(["--name", "a_dependent_job"])
       .assert().success();

    Ok(())
}

#[test]
fn two_queues() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("hstop")?;
    cmd.args(["--name", "two_queues"])
       .arg("--force").stderr(Stdio::piped()).status()?;

    let mut cmd = Command::cargo_bin("hstart")?;
    cmd.args(["--name", "two_queues"])
       .arg("1,1")
       .assert().success().stdout(predicate::str::contains("scheduler with nslots"));

    let mut cmd = Command::cargo_bin("hsubmit")?;
    cmd.args(["--name", "two_queues"])
       .arg("1,0");
    if env::consts::OS == "windows" {
        cmd.args(["powershell", "--", "-command", "sleep", "5"]);
    } else {
        cmd.args(["sleep", "5"]);
    }
    let mut child = cmd.stdout(Stdio::piped())
                       .spawn().unwrap();
    assert_stdout(&mut child, "1\n");

    let tmpdir = env::temp_dir();
    let mut path = PathBuf::from(&tmpdir);
    path.push("aitch");
    path.push("two_queues");

    path.push("secondqueue");
    let predicate_fn = predicate::path::is_file();
    assert_eq!(false, predicate_fn.eval(&path));

    let mut cmd = Command::cargo_bin("hsubmit")?;
    cmd.args(["--name", "two_queues"])
       .arg("0,1");
    if env::consts::OS == "windows" {
        let escaped_path = "'".to_string()+path.display().to_string().as_str()+"'";
        cmd.args(["powershell", "--", "-command", "ni", &escaped_path]);
    } else {
        cmd.args(["touch", path.display().to_string().as_str()]);
    }
    let mut child = cmd.stdout(Stdio::piped())
                       .spawn().unwrap();
    assert_stdout(&mut child, "2\n");

    thread::sleep(time::Duration::from_secs(1));
       
    let mut cmd = Command::cargo_bin("hjobs")?;
    cmd.args(["--name", "two_queues"])
       .assert().success().stdout(predicate::str::contains("\n").count(2));

    let mut cmd = Command::cargo_bin("hjobs")?;
    cmd.args(["--name", "two_queues"])
       .arg("running")
       .assert().success().stdout(predicate::str::contains("\n").count(2));

    let mut cmd = Command::cargo_bin("hjobs")?;
    cmd.args(["--name", "two_queues"])
       .arg("pending")
       .assert().success().stdout(predicate::str::contains("\n").count(1));

    let mut cmd = Command::cargo_bin("hjobs")?;
    cmd.args(["--name", "two_queues"])
       .arg("2");
    cmd.assert().failure().stderr(predicate::str::contains("no such job found"));

    let mut cmd = Command::cargo_bin("hjobs")?;
    cmd.args(["--name", "two_queues"])
       .arg("1");
    cmd.assert().success().stdout(predicate::str::contains("\n").count(2));

    wait_for_all_jobs_to_finish(cmd);

    let predicate_fn = predicate::path::is_file();
    assert_eq!(true, predicate_fn.eval(&path));
    path.pop();

    let mut cmd = Command::cargo_bin("hjobs")?;
    cmd.args(["--name", "two_queues"])
       .assert().success().stdout(predicate::str::contains("\n").count(1));

    let mut cmd = Command::cargo_bin("hstop")?;
    cmd.args(["--name", "two_queues"])
       .assert().success();

    Ok(())
}

#[test]
fn redirect() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("hstop")?;
    cmd.args(["--name", "redirect"])
       .arg("--force").stderr(Stdio::piped()).status()?;

    let mut cmd = Command::cargo_bin("hstart")?;
    cmd.args(["--name", "redirect"])
       .arg("1")
       .assert().success().stdout(predicate::str::contains("scheduler with nslots"));

    let tmpdir = env::temp_dir();
    let mut path = PathBuf::from(&tmpdir);
    path.push("aitch");
    path.push("redirect");

    path.push("redirect.out");  let outfile = path.display().to_string();  path.pop();
    path.push("redirect.err");  let errfile = path.display().to_string();  path.pop();

    let mut cmd = Command::cargo_bin("hsubmit")?;
    cmd.args(["--name", "redirect"])
       .args(["1", "--out", outfile.as_str(), "--err", errfile.as_str()]);
    if env::consts::OS == "windows" {
        cmd.args(["powershell", "--", "-command", "ls"]);
    } else {
        cmd.arg("ls");
    }
    let mut child = cmd.stdout(Stdio::piped())
                       .spawn().unwrap();
    assert_stdout(&mut child, "1\n");

    let mut cmd = Command::cargo_bin("hjobs")?;
    cmd.args(["--name", "redirect"])
       .arg("1");
    wait_for_all_jobs_to_finish(cmd);

    let predicate_fn = predicate::path::is_file();
    path.push("redirect.out");
    assert_eq!(true, predicate_fn.eval(&path));
    let outlen = path.metadata().unwrap().len();
    path.pop();
    path.push("redirect.err");
    assert_eq!(true, predicate_fn.eval(&path));
    let errlen = path.metadata().unwrap().len();
    path.pop();

    let mut cmd = Command::cargo_bin("hsubmit")?;
    cmd.args(["--name", "redirect"])
       .args(["1", "--out", outfile.as_str(), "--err", errfile.as_str()])
       .args(["--append"]);
    if env::consts::OS == "windows" {
        cmd.args(["powershell", "--", "-command", "ls"]);
    } else {
        cmd.arg("ls");
    }
    let mut child = cmd.stdout(Stdio::piped())
                       .spawn().unwrap();
    assert_stdout(&mut child, "2\n");

    let mut cmd = Command::cargo_bin("hjobs")?;
    cmd.args(["--name", "redirect"])
       .arg("2");
    wait_for_all_jobs_to_finish(cmd);

    let predicate_fn = predicate::path::is_file();
    path.push("redirect.out");
    assert_eq!(true, predicate_fn.eval(&path));
    assert_eq!(outlen*2, path.metadata().unwrap().len());
    path.pop();
    path.push("redirect.err");
    assert_eq!(true, predicate_fn.eval(&path));
    assert_eq!(errlen*2, path.metadata().unwrap().len());
    path.pop();

    let mut cmd = Command::cargo_bin("hstop")?;
    cmd.args(["--name", "redirect"])
       .assert().success();

    Ok(())
}

#[test]
fn envvar() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("hstop")?;
    cmd.args(["--name", "envvar"])
       .arg("--force").stderr(Stdio::piped()).status()?;

    let mut cmd = Command::cargo_bin("hstart")?;
    cmd.args(["--name", "envvar"])
       .arg("1")
       .assert().success().stdout(predicate::str::contains("scheduler with nslots"));

    let mut cmd = Command::cargo_bin("hsubmit")?;
    cmd.args(["--name", "envvar"])
       .args(["1", "--var", "FOO=foo", "--var", "BAR=bar"]);
    if env::consts::OS == "windows" {
        cmd.args(["powershell", "--", "-command", "dir", "env:"]);
    } else {
        cmd.arg("printenv");
    }
    let mut child = cmd.stdout(Stdio::piped())
                       .spawn().unwrap();
    assert_stdout(&mut child, "1\n");

    let mut cmd = Command::cargo_bin("hjobs")?;
    cmd.args(["--name", "envvar"])
       .arg("1");
    wait_for_all_jobs_to_finish(cmd);
       
    let tmpdir = env::temp_dir();
    let mut path = PathBuf::from(&tmpdir);
    path.push("aitch");
    path.push("envvar");

    path.push("1.out");
    let mut file = File::open(&path).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    if env::consts::OS == "windows" {
        assert!(contents.contains("FOO"));
        assert!(contents.contains("BAR"));
    } else {
        assert!(contents.contains("FOO=foo"));
        assert!(contents.contains("BAR=bar"));
    }

    let mut cmd = Command::cargo_bin("hstop")?;
    cmd.args(["--name", "envvar"])
       .assert().success();

    Ok(())
}

#[test]
fn envvar_with_islot() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("hstop")?;
    cmd.args(["--name", "envvar_with_islot"])
       .arg("--force").stderr(Stdio::piped()).status()?;

    let mut cmd = Command::cargo_bin("hstart")?;
    cmd.args(["--name", "envvar_with_islot"])
       .arg("3,2")
       .assert().success().stdout(predicate::str::contains("scheduler with nslots"));

    let mut cmd = Command::cargo_bin("hsubmit")?;
    cmd.args(["--name", "envvar_with_islot"])
       .arg("1,0");
    if env::consts::OS == "windows" {
        cmd.args(["powershell", "--", "-command", "sleep", "5"]);
    } else {
        cmd.args(["sleep", "5"]);
    }
    let mut child = cmd.stdout(Stdio::piped())
                       .spawn().unwrap();
    assert_stdout(&mut child, "1\n");

    let mut cmd = Command::cargo_bin("hsubmit")?;
    cmd.args(["--name", "envvar_with_islot"])
       .arg("1,2");
    if env::consts::OS == "windows" {
        cmd.args(["powershell", "--", "-command", "dir", "env:"]);
    } else {
        cmd.arg("printenv");
    }
    let mut child = cmd.stdout(Stdio::piped())
                       .spawn().unwrap();
    assert_stdout(&mut child, "2\n");

    thread::sleep(time::Duration::from_secs(1));

    let mut cmd = Command::cargo_bin("hjobs")?;
    cmd.args(["--name", "envvar_with_islot"])
       .arg("1");
    cmd.assert().success().stdout(predicate::str::contains("\n").count(2));
       
    wait_for_all_jobs_to_finish(cmd);

    let tmpdir = env::temp_dir();
    let mut path = PathBuf::from(&tmpdir);
    path.push("aitch");
    path.push("envvar_with_islot");

    path.push("2.out");
    let mut file = File::open(&path).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    if env::consts::OS == "windows" {
        assert!(contents.contains("QUEUE0"));
        assert!(contents.contains("QUEUE1"));
    } else {
        assert!(contents.contains("QUEUE0=1"));
        assert!(contents.contains("QUEUE1=0,1"));
    }

    let mut cmd = Command::cargo_bin("hstop")?;
    cmd.args(["--name", "envvar_with_islot"])
       .assert().success();

    Ok(())
}

#[test]
fn kill() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("hstop")?;
    cmd.args(["--name", "kill"])
       .arg("--force").stderr(Stdio::piped()).status()?;

    let mut cmd = Command::cargo_bin("hstart")?;
    cmd.args(["--name", "kill"])
       .arg("1")
       .assert().success().stdout(predicate::str::contains("scheduler with nslots"));

    let mut cmd = Command::cargo_bin("hsubmit")?;
    cmd.args(["--name", "kill"])
       .arg("1");
    if env::consts::OS == "windows" {
        cmd.args(["powershell", "--", "-command", "sleep", "5"]);
    } else {
        cmd.args(["sleep", "5"]);
    }
    let mut child = cmd.stdout(Stdio::piped())
                       .spawn().unwrap();
    assert_stdout(&mut child, "1\n");

    thread::sleep(time::Duration::from_secs(1));
       
    let mut cmd = Command::cargo_bin("hjobs")?;
    cmd.args(["--name", "kill"])
       .assert().success().stdout(predicate::str::contains("\n").count(2));

    let mut cmd = Command::cargo_bin("hkill")?;
    cmd.args(["--name", "kill", "--kill"])
       .arg("1")
       .assert().success();

    let mut cmd = Command::cargo_bin("hjobs")?;
    cmd.args(["--name", "kill"])
       .assert().success().stdout(predicate::str::contains("\n").count(1));

    let mut cmd = Command::cargo_bin("hstop")?;
    cmd.args(["--name", "kill"])
       .assert().success();

    Ok(())
}

#[test]
fn two_schedulers() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("hstop")?;
    cmd.args(["--name", "first_scheduler", "--name", "second_scheduler"])
       .arg("--force").stderr(Stdio::piped()).status()?;

    let mut cmd = Command::cargo_bin("hstart")?;
    cmd.args(["--name", "first_scheduler"])
       .assert().success().stdout(predicate::str::contains("scheduler with nslots"));

    let mut cmd = Command::cargo_bin("hstatus")?;
    cmd.args(["--name", "first_scheduler"])
       .assert().success().stdout(predicate::str::contains("\n").count(2));

    let mut cmd = Command::cargo_bin("hstatus")?;
    cmd.args(["--name", "second_scheduler"])
       .assert().failure().stderr(predicate::str::contains("bad state or not running"));

    let mut cmd = Command::cargo_bin("hstart")?;
    cmd.args(["--name", "second_scheduler"])
       .assert().success().stdout(predicate::str::contains("scheduler with nslots"));

    let mut cmd = Command::cargo_bin("hstatus")?;
    cmd.args(["--name", "first_scheduler", "--name", "second_scheduler"])
       .assert().success().stdout(predicate::str::contains("\n").count(3));

    let mut cmd = Command::cargo_bin("hstatus")?;
    cmd.args(["--name", "second_scheduler"])
       .assert().success().stdout(predicate::str::contains("\n").count(2));

    let mut cmd = Command::cargo_bin("hstop")?;
    cmd.args(["--name", "first_scheduler"])
       .assert().success();

    let mut cmd = Command::cargo_bin("hstatus")?;
    cmd.args(["--name", "first_scheduler"])
       .assert().failure().stderr(predicate::str::contains("bad state or not running"));

    let mut cmd = Command::cargo_bin("hstatus")?;
    cmd.args(["--name", "second_scheduler"])
       .assert().success().stdout(predicate::str::contains("\n").count(2));

    let mut cmd = Command::cargo_bin("hstop")?;
    cmd.args(["--name", "second_scheduler"])
       .assert().success();

    let mut cmd = Command::cargo_bin("hstatus")?;
    cmd.args(["--name", "second_scheduler"])
       .assert().failure().stderr(predicate::str::contains("bad state or not running"));

    Ok(())
}
