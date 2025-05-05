use anyhow::Result;
use clap::{Parser, Subcommand, ValueHint};
use uuid::Uuid;

use todo::model::Task;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Cli {
    /// Main verb. If omitted, `list` is default action.
    #[command(subcommand)]
    verb: Option<Verb>,

    /// Shared flag for all verbs.
    #[arg(short = 't', long, default_value = "Task")]
    title: String,

    #[arg(short, long, value_hint = ValueHint::FilePath, default_value = "./todo.json")]
    output: String,
}

#[derive(Subcommand, Debug)]
enum Verb {
    Add {
        desc: String,

        #[arg(short = 'p', long)]
        priority: Option<u8>,

        #[arg(long, value_delimiter = ',')]
        tags: Vec<String>,
    },
    Complete {
        #[arg(short = 'c', long)]
        id: Uuid,
    },
    Remove {
        #[arg(short = 'r', long)]
        id: Uuid,
    },
    List {
        #[arg(short = 'p', long)]
        priority: Option<u8>,

        #[arg(long, value_delimiter = ',')]
        tags: Vec<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.verb.unwrap_or(Verb::List {
        priority: None,
        tags: Vec::new(),
    }) {
        Verb::Add {
            desc,
            priority,
            tags,
        } => {
            let task = Task::builder()
                .title(desc)
                .priority(priority.unwrap_or(0))
                .tags
                .into_iter()
                .fold(Task::builder().title(&cli.title), |b, tag| b.tag(tag))
                .build();

            todo::add_task(&cli.output, task)?;
        }
        Verb::Complete { id } => todo::complete_task(&cli.output, id)?,
        Verb::Remove { id } => todo::remove_task(&cli.output, id)?,
        Verb::List { priority, tags } => todo::list_tasks(&cli.output, priority, &tags)?,
    }
    Ok(())
}
