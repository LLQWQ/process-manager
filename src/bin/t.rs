use std::{
    ffi::OsString,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use sysinfo::System;

fn main() {
    let pid = 1449982; // 替换为要查询的 PID
    let mut system = System::new_all();
    system.refresh_all();

    if let Some(process) = system.process(sysinfo::Pid::from_u32(pid)) {
        let cmd = process.cmd();
        let join = cmd.join(&OsString::from(" ")).into_string().unwrap();

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

        println!("cmd: {:?}", join);
        println!("Process start time: {:?}", formatted_time);
        println!("Execution duration: {} seconds", execution_duration);
    } else {
        println!("Process with PID {} not found.", pid);
    }
}
