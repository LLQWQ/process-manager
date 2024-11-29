use std::{
    error::Error,
    io::{self, BufWriter, Write},
    process::Command,
};

use clap::{Args, Parser, Subcommand};
use dialoguer::{theme::ColorfulTheme, Confirm, Editor, Input, MultiSelect, Select};
use duct::cmd;
use prettytable::{row, Table};
use process_manager::{process, ProcessItem, ProcessManager, SearchArgs};

/// Simple program to greet a person
#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    List(SearchArgs),
    Add(AddArgs),
    Rm(SearchArgs),
    Start(SearchArgs),
    Stop(SearchArgs),
}

#[derive(Args)]
pub struct AddArgs {
    /// 从pid导入信息
    #[arg(long)]
    pid: Option<u32>,
    /// 标签
    #[arg(short, long)]
    tags: Option<Vec<String>>,
    /// 进程唯一名称
    #[arg(short, long)]
    name: Option<String>,
    /// 执行命令
    #[arg(short, long)]
    command: Option<String>,
    /// 进程类型
    #[arg(short, long)]
    process_type: Option<String>,
    /// 日志输出文件地址
    #[arg(short, long)]
    log_path: Option<String>,
    /// 健康检测命令
    #[arg(long)]
    detection_start_cmd: Option<String>,
    /// pid查询命令
    #[arg(long)]
    pid_search_cmd: Option<String>,
    /// 备注
    #[arg(long)]
    comment: Option<String>,
}

fn main() {
    /*
    pm [command]
    # group [args]
    ## list add rm update
    # add [args]
    ## --name hello [--group DEFAULT_GROUP] --cmd "java -jar xxx.jar" --log "/xx/xx.log" --type java --
    ## --name hello --pid Vec<pid>
    ##
     */
    let cli = Cli::parse();
    //~/.process_manager
    let mut pm = ProcessManager::new(".");

    match cli.command {
        Commands::List(search_args) => {
            let processes = pm.list(search_args);
            print_processes(processes)
        }
        Commands::Add(add_args) => {
            let pi = build_process_item(add_args);
            pm.add(pi)
        }
        Commands::Rm(search_args) => {
            let processes = pm.list(search_args);
            let collect = processes
                .iter()
                .map(|it| it.name.clone())
                .collect::<Vec<_>>();
            pm.remove(collect)
        }
        Commands::Start(search_args) => {
            let processes = pm.list(search_args);
            let collect = processes
                .iter()
                .map(|it| it.name.clone())
                .collect::<Vec<_>>();
            pm.start(collect)
        }
        Commands::Stop(search_args) => {}
    }

    // cmd!("sleep", "100")
    //     .before_spawn(|cmd| {
    //         prevent_being_killed(cmd)?;
    //         Ok(())
    //     })
    //     .start()
    //     .unwrap()
    //     .wait()
    //     .unwrap();

    // let template_options = vec!["Vue", "React", "Svelte"];
    // let selection = Select::with_theme(&ColorfulTheme::default())
    //     .with_prompt("Select a project template")
    //     .items(&template_options)
    //     .default(0)
    //     .interact()
    //     .unwrap();

    // println!("{}", template_options[selection].to_string());
}
fn print_processes(processes: Vec<&ProcessItem>) {
    // Create the table
    let mut table = Table::new();

    // Add a row per time
    table.add_row(row![
        "唯一程序名",
        "启动命令",
        "标签",
        "进程类型",
        "日志路径",
        "检测启动命令",
        "备注"
    ]);

    // 打印每个进程的信息
    for process in processes {
        table.add_row(row![
            process.name,
            process.command,
            process.tags.join(", "),
            process.process_type,
            process.log_path,
            process.detection_start_cmd,
            process.comment,
        ]);
    }

    table.printstd();
}

fn build_process_item(mut add_args: AddArgs) -> ProcessItem {
    // 获取pid对应的命令
    if let Some(pid) = add_args.pid {
        if let Some((cmd, start_time, execution_duration)) = process::get_process_info(pid) {
            add_args.command = Some(cmd.clone());
            println!(
                "pid:{}\t执行命令:{}\t启动时间:{}\t执行时长:{}s",
                pid, cmd, start_time, execution_duration
            );
        } else {
            println!("未找到pid:{}对应的进程", pid);
        }
    }

    // 输入必填字段
    loop {
        if add_args.name.is_none() {
            add_args.name = Some(
                Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("输入唯一程序名")
                    .interact_text()
                    .unwrap(),
            );
        }

        if add_args.command.is_none() {
            add_args.command = Some(
                Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("输入启动命令")
                    .interact_text()
                    .unwrap(),
            );
        }

        // 输出当前属性
        println!(
            r#"
唯一程序名：{}
启动命令：{}
标签：{:?}
进程类型：{}
日志路径：{}
检测启动命令：{}
备注：{}
"#,
            add_args.name.as_ref().unwrap(),
            add_args.command.as_ref().unwrap(),
            add_args.tags.as_ref().unwrap_or(&vec![]),
            add_args.process_type.as_ref().unwrap_or(&"".to_string()),
            add_args.log_path.as_ref().unwrap_or(&"".to_string()),
            add_args
                .detection_start_cmd
                .as_ref()
                .unwrap_or(&"".to_string()),
            add_args.comment.as_ref().unwrap_or(&"".to_string())
        );

        // 询问是否需要修改
        if !Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("还有要修改的嘛?")
            .default(false)
            .wait_for_newline(true)
            .interact()
            .unwrap()
        {
            break;
        }

        // 提供修改选项
        let modify_options = &[
            format!("唯一程序名：{}", add_args.name.as_ref().unwrap()),
            format!("启动命令：{}", add_args.command.as_ref().unwrap()),
            format!("标签：{:?}", add_args.tags.as_ref().unwrap_or(&vec![])),
            format!(
                "进程类型：{}",
                add_args.process_type.as_ref().unwrap_or(&"".to_string())
            ),
            format!(
                "日志路径：{}",
                add_args.log_path.as_ref().unwrap_or(&"".to_string())
            ),
            format!(
                "检测启动命令：{}",
                add_args
                    .detection_start_cmd
                    .as_ref()
                    .unwrap_or(&"".to_string())
            ),
            format!(
                "备注：{}",
                add_args.comment.as_ref().unwrap_or(&"".to_string())
            ),
        ];
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("选择需要修改的值")
            .items(&modify_options[..])
            .interact()
            .unwrap();

        match selection {
            0 => {
                add_args.name = Some(
                    Input::<String>::with_theme(&ColorfulTheme::default())
                        .with_prompt("输入新的唯一程序名")
                        .with_initial_text(add_args.name.clone().unwrap())
                        .interact_text()
                        .unwrap(),
                );
            }
            1 => {
                add_args.command = Some(
                    Input::<String>::with_theme(&ColorfulTheme::default())
                        .with_prompt("输入新的启动命令")
                        .with_initial_text(add_args.command.clone().unwrap())
                        .interact_text()
                        .unwrap(),
                );
            }
            2 => {
                add_args.tags = Some(
                    Input::<String>::with_theme(&ColorfulTheme::default())
                        .with_prompt("输入新的标签 (用逗号分隔)")
                        .with_initial_text(
                            add_args
                                .tags
                                .as_ref()
                                .map(|tags| tags.join(", "))
                                .unwrap_or_default(),
                        )
                        .interact_text()
                        .unwrap()
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .collect(),
                );
            }
            3 => {
                add_args.process_type = Some(
                    Input::<String>::with_theme(&ColorfulTheme::default())
                        .with_prompt("输入新的进程类型")
                        .with_initial_text(add_args.process_type.clone().unwrap_or_default())
                        .interact_text()
                        .unwrap(),
                );
            }
            4 => {
                add_args.log_path = Some(
                    Input::<String>::with_theme(&ColorfulTheme::default())
                        .with_prompt("输入新的日志路径")
                        .with_initial_text(add_args.log_path.clone().unwrap_or_default())
                        .interact_text()
                        .unwrap(),
                );
            }
            5 => {
                add_args.detection_start_cmd = Some(
                    Input::<String>::with_theme(&ColorfulTheme::default())
                        .with_prompt("输入新的检测启动命令")
                        .with_initial_text(add_args.detection_start_cmd.clone().unwrap_or_default())
                        .interact_text()
                        .unwrap(),
                );
            }
            6 => {
                add_args.comment = Some(
                    Input::<String>::with_theme(&ColorfulTheme::default())
                        .with_prompt("输入新的备注")
                        .with_initial_text(add_args.comment.clone().unwrap_or_default())
                        .interact_text()
                        .unwrap(),
                );
            }
            _ => unreachable!(),
        }
    }

    // 创建并返回ProcessItem
    ProcessItem {
        name: add_args.name.unwrap(),
        tags: add_args.tags.unwrap_or(vec![]),
        command: add_args.command.unwrap(),
        process_type: add_args.process_type.unwrap_or_else(|| "".to_string()),
        log_path: add_args.log_path.unwrap_or_else(|| "".to_string()),
        detection_start_cmd: add_args
            .detection_start_cmd
            .unwrap_or_else(|| "".to_string()),
        comment: add_args.comment.unwrap_or_else(|| "".to_string()),
    }
}
