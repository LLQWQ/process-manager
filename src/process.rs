use std::{
    ffi::OsString,
    fs::File,
    io,
    path::Path,
    process::Command,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use chrono::DateTime;
use duct::cmd;
use sysinfo::System;

pub fn get_process_info(pid: u32) -> Option<(String, String, u64)> {
    let mut system = System::new_all();
    system.refresh_all();

    system.process(sysinfo::Pid::from_u32(pid)).map(|process| {
        let cmd = process
            .cmd()
            .join(&OsString::from(" "))
            .into_string()
            .unwrap();

        // 获取进程启动时间（单位: 秒，UNIX 时间戳）
        let start_time_seconds = process.start_time();
        let start_time = UNIX_EPOCH + Duration::from_secs(start_time_seconds);
        // DateTime::from_timestamp
        // 格式化启动时间为 yyyy-MM-dd HH:mm:ss
        let formatted_time = DateTime::from_timestamp(start_time_seconds as i64, 0)
            .unwrap()
            .naive_local()
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();

        // 计算执行时长
        let now = SystemTime::now();
        let execution_duration = now.duration_since(start_time).unwrap().as_secs();

        (cmd, formatted_time, execution_duration)

        // println!("cmd: {:?}", join);
        // println!("Process start time: {:?}", formatted_time);
        // println!("Execution duration: {} seconds", execution_duration);
    })

    // else {
    //     println!("Process with PID {} not found.", pid);
    // }
}

pub fn is_started(detection_start_cmd: &str) -> io::Result<bool> {
    let result = cmd!("bash", "-c", detection_start_cmd).read()?;

    Ok(!result.is_empty())
}

pub fn swpan(command: &str, log_path: &str) {
    cmd!("bash", "-c", command)
        .before_spawn(|cmd| {
            prevent_being_killed(cmd)?;
            Ok(())
        })
        .stderr_to_stdout()
        .stdout_path(log_path)
        .start()
        .unwrap()
        .wait()
        .unwrap();
}

fn prevent_being_killed(cmd: &Command) -> io::Result<()> {
    nix::unistd::daemon(true, true)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    Ok(())
}
