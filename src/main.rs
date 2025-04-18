use anyhow::Result;
use clap::{Parser, Subcommand, ValueHint};
use std::path::PathBuf;
use time::UtcDateTime;
use todo::TodoConfig;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Cli {
    /// Main verb. If omitted, `list` is default action.
    #[command(subcommand, default_value_t = Verb::List)]
    verb: Verb,

    /// Shared flag for all verbs.
    #[arg(short = 't', long = "title", default_value = "Task")]
    title: String,

    #[arg(short, long, value_hint = ValueHint::FilePath, default_value = "./todo.json")]
    output: PathBuf,
}

#[derive(Subcommand, Debug)]
enum Verb {
    Add {
        task: String,
    },
    Complete {
        #[arg(short = 'c', long)]
        id: i32,
    },
    List,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.verb {
        Verb::Add { task } => add_task(&cli.output, task)?,
        Verb::Complete { id } => complete_task(&cli.output, id)?,
        Verb::List => list_tasks(&cli.output)?,
    }
    Ok(())

    //todo::write_task_to_file(&cfg, task);
    //let cfg = TodoConfig { output: cli.output };
    //let todos: Vec<todo::Todo> = todo::read_tasks_from_file(&cfg);
    //let todo = todo::todo::new(&task[..], &title[..]);
}
