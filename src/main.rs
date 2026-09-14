use std::fmt::Debug;
use std::io::Write;
use std::fmt::Write as OtherWrite;
use std::path::{Path, PathBuf};

use chrono::Local;
use clap::builder::styling::{self};
use clap::{ArgAction, Parser, Subcommand};

fn main() {
    let cli = Cli::parse();
    match cli.action {
        CliAction::List { 
            filter,
            exclude_tagged,
            exclude_untagged,
            include_tags,
            exclude_tags,
            list_all 
        } => {

            let todos = get_todos();

            let mut query: Vec<&Todo> = todos.iter()
                .filter(|todo| {
                    // filter todos that are open
                    list_all || todo.status == TodoStatus::Open
                })
                .filter(|todo| {
                    // filter todos that contain query in title
                    filter.as_ref().is_none_or(
                        |it| todo.title.contains(it.as_str())
                    )
                })
                .filter(|todo| {
                    // filter todos that are untagged
                    !exclude_tagged || todo.tags.is_empty()
                })
                .filter(|todo| {
                    // filter todos that are tagged
                    !exclude_untagged || !todo.tags.is_empty()
                })
                .filter(|todo| {
                    // filter todos that contain tag
                    // the todo must contain at least one tag of each group
                    include_tags.as_ref().is_none_or(|groups| {
                        groups.iter().all(|group| {
                            group.iter().any(|tag| todo.tags.contains(tag))
                        })
                    })
                })
                .filter(|todo| {
                    // exclude todos that do not contain tags
                    exclude_tags.as_ref().is_none_or(|groups| {
                        groups.iter().any(|group| {
                            !group.iter().any(|tag| todo.tags.contains(tag))
                        })
                    })
                })
                .collect();

            query.sort_by_key(|todo| todo.path_name());
            query.sort_by_key(|todo| todo.rank);

            println!();

            for todo in query.iter().rev() {
                list_todo(todo);
            }

            println!();
        }
        CliAction::Add {title, mut tags, rank} => {
            tags.sort();

            let mut todo = Todo::new(
                None,
                title,
                tags,
                rank,
                Some(TodoStatus::Open),
            );
            
            put_todo(&mut todo);
            println!();
            list_todo(&todo);
            println!();
        },
        CliAction::Close { path } => {

            let mut todo = get_todo(path).expect("could not find todo");

            todo.status = TodoStatus::Done;

            patch_todo(&todo);

            println!();
            list_todo(&todo);
            println!();
        }
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    action: CliAction,
}

#[derive(Subcommand, Debug)]
enum CliAction {
    Add {
        title: String,

        #[arg(short, long, action=ArgAction::Append, num_args=1..)]
        tags: Vec<String>,

        #[arg(short, long)]
        rank: Option<u32>
    },
    Close {
        path: String
    },
    List {
        #[arg(short, long)]
        filter: Option<String>,

        #[arg(short='a')]
        list_all: bool,

        #[arg(short='u', long)]
        exclude_tagged: bool,

        #[arg(short='U', long)]
        exclude_untagged: bool,

        #[arg(short='t', long, action=ArgAction::Append, num_args=1..)]
        include_tags: Option<Vec<Vec<String>>>,
        
        #[arg(short='T', long, action=ArgAction::Append, num_args=1..)]
        exclude_tags: Option<Vec<Vec<String>>>,
    },
}

#[derive(Debug)]
struct Todo {
    path: Option<PathBuf>,
    rank: u32,
    title: String,
    status: TodoStatus,
    tags: Vec<String>
}

#[derive(Debug, PartialEq)]
enum TodoStatus {
    Open,
    Done,
}

impl Todo {
    fn new(
        path: Option<PathBuf>,
        title: String,
        tags: Vec<String>,
        rank: Option<u32>,
        status: Option<TodoStatus>,
    ) -> Self {
        const DEFAULT_STATUS: TodoStatus = TodoStatus::Open;
        const DEFAULT_RANK: u32 = 0; 
        Self {
            path,
            title,
            tags,
            rank: rank.unwrap_or(DEFAULT_RANK),
            status: status.unwrap_or(DEFAULT_STATUS)
        }
    }

    fn path_name(&self) -> &str {
        self.path.as_ref()
            .unwrap()
            .to_str()
            .unwrap_or("invalid path")
    }
}

/// Writes a `Todo` to the file system
fn put_todo(todo: &mut Todo) {
    let path = todo.path.get_or_insert({
        const TASK_DIR: &str = "tasks";
        let id = generate_todo_id();
        format!("{}/{}.md", TASK_DIR, id).into()
    });

    std::fs::create_dir_all("tasks").unwrap();
    let file = std::fs::File::options()
        .write(true)
        .create_new(true)
        .open(path)
        .expect("Could not create file");

    put_todo_write(file, &todo).unwrap();
}

fn put_todo_write<W: Write>(mut buf: W, todo: &Todo) -> std::io::Result<()> {
    writeln!(&mut buf, "---")?;
    writeln!(&mut buf, "title: {}", todo.title)?;
    writeln!(&mut buf, "status: {}", match todo.status {
        TodoStatus::Open => "open",
        TodoStatus::Done => "done",
    })?;
    writeln!(&mut buf, "rank: {}", todo.rank)?;
    writeln!(&mut buf, "tags: [ {} ]", todo.tags.join(", "))?;
    writeln!(&mut buf, "---")?;
    Ok(())
}

fn patch_todo(todo: &Todo) {
    let path = todo.path.as_ref().unwrap();

    let content = std::fs::read_to_string(path).unwrap();
    let (frontmatter, body) = split_frontmatter(&content).unwrap();

    let patched_frontmatter = frontmatter.lines()
        .map(|line| {
            if let Some((key, _)) = line.split_once(':') {
                match key.trim() {
                    "rank" => {
                        format!("rank: {}", todo.rank)
                    }
                    "title" => {
                        format!("title: {}", todo.title)
                    }
                    "status" => {
                        format!("status: {}", match todo.status {
                            TodoStatus::Open => "open",
                            TodoStatus::Done => "done",
                        })
                    }
                    "tags" => {
                        format!("tags: [{}]", todo.tags.join(", "))
                    }
                    _ => String::from(line)
                }
            }
            else { String::from(line) }
        })
        .collect::<Vec<String>>()
        .join("\n");

    std::fs::create_dir_all("tasks").unwrap();

    let file = std::fs::File::options()
        .write(true)
        .open(path)
        .expect("Could not create file");

    patch_todo_write(file, &patched_frontmatter, body)
        .expect("failed to write");
}

fn patch_todo_write<W: Write>(
    mut buf: W, 
    frontmatter: &str, 
    body: &str
) -> std::io::Result<()> {
    write!(&mut buf, "---\n")?;
    write!(&mut buf, "{}\n", frontmatter)?;
    write!(&mut buf, "---\n")?;
    write!(&mut buf, "{}", body)?;
    Ok(())
}

fn get_todo<P: AsRef<Path>>(path: P) -> Option<Todo> {
    let s = std::fs::read_to_string(&path).ok()?;
    let mut todo = parse_todo(&s)?;
    todo.path.replace(path.as_ref().to_path_buf());
    Some(todo)
}

fn get_todos() -> Vec<Todo> {
    let mut todos = Vec::new();

    if std::fs::exists("tasks").unwrap() {
        for entry in std::fs::read_dir("tasks").unwrap() {
            let path = entry.unwrap().path();
            if path.is_file() {
                if let Some(todo) = get_todo(&path) {
                    todos.push(todo)
                }
                else {
                    eprintln!("ignoring invalid task file `{}`", path.to_str().unwrap())
                }
            }
        }
    }

    todos 
}

fn list_todo(todo: &Todo) {

    let mut tags = todo.tags.iter()
        .map(|it| format!("({it})"))
        .collect::<Vec<String>>();
    tags.sort();

    println!(
        "{}<{}>   {:>3}   {:.<32}   {}",
        
        match todo.status {
            TodoStatus::Done => styling::AnsiColor::Green,
            TodoStatus::Open => styling::AnsiColor::Black
        }.render_fg(),
        todo.path_name(),
        // styling::Reset.render(),
        todo.rank,
        todo.title.as_str(),
        tags.join(" ")
    )
}

fn generate_todo_id() -> String {
    let dt = Local::now();
    let mut id = String::new();
    write!(&mut id, "{}", dt.format("%y%m%d-%H%M%S")).unwrap();
    id
}

fn split_frontmatter(s: &str) -> Option<(&str, &str)> {
    const DELIM: &str = "---\n";
    s.strip_prefix(DELIM)?.split_once(DELIM)
}

fn parse_todo(s: &str) -> Option<Todo> {
    

    // let mut id: Option<u32> = None;
    let mut title: Option<&str> = None;
    let mut status: Option<TodoStatus> = None;
    let mut rank: Option<u32> = None;
    let mut tags: Vec<String> = Vec::new();

    let (frontmatter, _) = split_frontmatter(s)?;

    for line in frontmatter.lines() {
        if let Some((key, value)) = line.trim().split_once(':') {
            match key.trim() {
                "rank" => {
                    rank.replace(u32::from_str_radix(value.trim(), 10).ok()?);
                }
                "title" => {
                    title.replace(value.trim());
                },
                "tags" => {
                    tags.extend(value.trim()
                        .strip_prefix('[')?
                        .strip_suffix(']')?
                        .split(',')
                        .map(|tag| tag.trim().to_string())
                        .filter(|tag| !tag.is_empty())
                    );
                },
                "status" => {
                    match value.trim() {
                        "open" => status.replace(TodoStatus::Open),
                        "done" => status.replace(TodoStatus::Done),
                        _ => { return None; }
                    };
                },
                _ => {}
            }
        }
    }

    Some(Todo::new(
        None,
        title?.to_string(),
        tags,
        rank,
        status
    ))
}
