pub mod process;

use clap::{arg, Args, Command};
use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    fs::OpenOptions,
    io::{self, BufWriter, Read, Write},
    time::SystemTime,
};

pub struct ProcessManager {
    conf_path: String,
    conf: Conf,
}

#[derive(Serialize, Deserialize)]
struct Conf {
    processes: Vec<ProcessItem>,
}

#[derive(Serialize, Deserialize)]
pub struct ProcessItem {
    pub name: String,
    pub tags: Vec<String>,
    pub command: String,
    pub process_type: String,
    pub log_path: String,
    /// 检测启动命令
    pub detection_start_cmd: String,
    pub comment: String,
}
impl ProcessItem {
    fn format(&self) -> String {
        format!(
            "{}\t{}\t{}\t{}\t{}\t{}",
            self.process_type,
            self.name,
            self.tags.join(", "),
            self.command,
            self.detection_start_cmd,
            self.log_path
        )
    }
}

#[derive(Args)]
pub struct SearchArgs {
    #[arg(short, long)]
    tags: Option<Vec<String>>,
    #[arg(short, long)]
    name: Option<Vec<String>>,
    #[arg(short, long)]
    command: Option<Vec<String>>,
    #[arg(short, long)]
    process_type: Option<Vec<String>>,
    #[arg(short, long)]
    log_path: Option<Vec<String>>,
    #[arg(short, long)]
    detection_start_cmd: Option<Vec<String>>,
}

impl ProcessManager {
    pub fn new(profile_path: &str) -> Self {
        let conf_path = format!("{}/.config.json", profile_path);
        ProcessManager {
            conf_path: conf_path.clone(),
            conf: ProcessManager::load_conf(conf_path.as_str()),
        }
    }

    pub fn list(&self, search_args: SearchArgs) -> Vec<&ProcessItem> {
        self.conf
            .processes
            .iter()
            .filter(|it| ProcessManager::filter(it, &search_args))
            .collect()
        // let write_data = self
        //     .conf
        //     .processes
        //     .iter()
        //     .filter_map(|it| {
        //         if ProcessManager::filter(it, &search_args) {
        //             Some(it.format())
        //         } else {
        //             None
        //         }
        //     })
        //     .collect::<Vec<String>>()
        //     .join("\n");

        // let stdout = io::stdout();
        // let mut writer = BufWriter::new(stdout);

        // writer
        //     .write_all(write_data.as_bytes())
        //     .and_then(|_| writer.flush())
        //     .expect("输出控制台失败");
    }

    fn filter(ele: &ProcessItem, search_args: &SearchArgs) -> bool {
        let mut filter = true;

        if let Some(ref command) = search_args.command {
            filter = command.contains(&ele.command);
        }
        if let Some(ref tags) = search_args.tags {
            filter = filter && tags.iter().any(|it| ele.tags.contains(it));
        }
        if let Some(ref name) = search_args.name {
            filter = filter && name.contains(&ele.name);
        }
        if let Some(ref process_type) = search_args.process_type {
            filter = filter && process_type.contains(&ele.process_type);
        }
        if let Some(ref log_path) = search_args.log_path {
            filter = filter && log_path.contains(&ele.log_path);
        }
        if let Some(ref detection_start_cmd) = search_args.detection_start_cmd {
            filter = filter && detection_start_cmd.contains(&ele.log_path);
        }

        filter
    }

    pub fn add(&mut self, process_item: ProcessItem) {
        self.conf.processes.push(process_item);

        self.rewrite();
    }

    fn load_conf(conf_path: &str) -> Conf {
        // 检查文件是否存在
        if !std::path::Path::new(conf_path).exists() {
            println!("File does not exist. Creating file with initial content.");
            let initial_content = r#"{
                "processes": []
            }"#;
            // 创建文件并写入初始值
            std::fs::File::create(conf_path)
                .and_then(|mut file| file.write_all(initial_content.as_bytes()))
                .expect("创建初始配置文件失败");
        }

        let data = OpenOptions::new()
            .read(true)
            .open(conf_path)
            .and_then(|mut conf_file| {
                let mut data = String::new();
                conf_file.read_to_string(&mut data)?;
                Ok(data)
            })
            .expect("读取配置文件失败");
        let config: Conf = json5::from_str(&data).expect("转换配置错误");
        config
        /*
            // 反序列化 JSON5
        let config: Config = json5::from_str(json5_data)?;
        println!("Deserialized: {:?}", config);

        // 序列化回 JSON5
        let serialized = json5::to_string(&config)?;
        println!("Serialized: {}", serialized);
             */
    }

    fn rewrite(&self) {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&self.conf_path)
            .expect(&format!("打开配置文件失败：{}", self.conf_path));

        let serialized = json5::to_string(&self.conf).expect("序列化配置失败");

        file.write_all(serialized.as_bytes())
            .and_then(|_| file.flush())
            .expect(&format!("写入配置失败: {}", serialized));
    }

    pub fn remove(&mut self, names: Vec<String>) {
        self.conf.processes.retain(|it| !names.contains(&it.name));

        self.rewrite();
    }

    pub fn start(&self, collect: Vec<String>) {
        let filter = self
            .conf
            .processes
            .iter()
            .filter(|it| collect.contains(&it.name));
        for ele in filter {
            // 检测是否已启动
            if process::is_started(&ele.detection_start_cmd).expect("检查是否启动命令有误")
            {
                continue;
            }

            process::swpan(&ele.command, &ele.log_path);
        }
    }
}

mod tests {
    use std::env;

    use crate::{ProcessItem, ProcessManager, SearchArgs};

    #[test]
    fn test_load() {
        let cwd = env::current_dir().unwrap();
        println!("Current directory: {}", cwd.display());

        let load_conf = ProcessManager::load_conf("./.config.json");
    }

    #[test]
    fn test_list() {
        let mut pm = ProcessManager::new(".");
        pm.list(SearchArgs {
            tags: None,
            name: None,
            command: None,
            process_type: None,
            detection_start_cmd: None,
            log_path: None,
        });
    }

    #[test]
    fn test_add() {
        let mut pm = ProcessManager::new(".");
        pm.add(ProcessItem {
            tags: vec![],
            name: "hello".to_string(),
            command: "java -jar xxx.jar".to_string(),
            process_type: "java".to_string(),
            log_path: "{PM_PATH}/process_log/{name}".to_string(),
            detection_start_cmd: "dscmd".to_string(),
            comment: "备注".to_string(),
        });
        pm.list(SearchArgs {
            tags: None,
            name: None,
            command: None,
            process_type: None,
            detection_start_cmd: None,
            log_path: None,
        });
    }
}
