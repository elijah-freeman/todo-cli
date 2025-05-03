use anyhow::Result;
use clap::{Parser, Subcommand, ValueHint};

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Cli {
    /// Main verb. If omitted, `list` is default action.
    #[command(subcommand)]
    verb: Option<Verb>,

    /// Shared flag for all verbs.
    #[arg(short = 't', long = "title", default_value = "Task")]
    title: String,

    #[arg(short, long, value_hint = ValueHint::FilePath, default_value = "./todo.json")]
    output: String,
}

#[derive(Subcommand, Debug)]
enum Verb {
    Add {
        desc: String,

        #[arg(short = 'p')]
        priority: Option<u8>,

        #[arg(long)]
        tags: Option<Vec<String>>,
    },
    Complete {
        #[arg(short = 'c', long)]
        id: u32,
    },
    Remove {
        #[arg(short = 'r', long)]
        id: u32,
    },
    List {
        #[arg(short = 'p')]
        priority: Option<u8>,

        #[arg(long)]
        tags: Option<Vec<String>>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.verb.unwrap_or(Verb::List {
        priority: None,
        tags: None,
    }) {
        Verb::Add {
            desc,
            priority,
            tags,
        } => todo::add_task(&cli.output, &desc, &cli.title, priority, tags)?,
        Verb::Complete { id } => todo::complete_task(&cli.output, id)?,
        Verb::Remove { id } => todo::remove_task(&cli.output, id)?,
        Verb::List { priority, tags } => todo::list_tasks(&cli.output, priority, tags)?,
    }
    Ok(())

    //let todos: Vec<todo::Todo> = todo::read_tasks_from_file(&cfg);
    //let todo = todo::todo::new(&task[..], &title[..]);
}
