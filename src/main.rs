use anyhow::Result;
use clap::{Parser, ValueHint};
use std::path::PathBuf;
use time::UtcDateTime;
use todo::TodoConfig;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Cli {
    #[arg(short = 't', long = "title", default_value = "Task")]
    title: String,

    task: Option<String>,

    #[arg(short, long, value_hint = ValueHint::FilePath, default_value = "./todo.json")]
    output: PathBuf,

    #[arg(short = 'c', long = "complete")]
    index: Option<i32>,
}

fn main() {
    let cli = Cli::parse();
    let cfg = TodoConfig {
        title: cli.title,
        task: cli.task.expect("a todo list task"),
        output: cli.output,
        index: cli.index,
    };

    let title = &cfg.title;
    let task = &cfg.task;

    let todo = todo::todo::new(&task[..], &title[..]);

    todo::write_task_to_file(&cfg, &todo);
}
