use postgres::{Client, NoTls};
use std::collections::VecDeque;

extern crate dotenv;
use dotenv::dotenv;
use std::env;

struct Todo {
    id: i32,
    title: String,
    content: String,
    author: String,
    last_edit_by: String,
    created_at: chrono::NaiveTime,
    updated_at: chrono::NaiveTime,
    deleted_at: chrono::NaiveTime,
    state: i32,
}

struct TodoFieldDescribe {
    todo: String,
    kv: [(String, String)],
}

struct CreateTodo {
    title: String,
    author: String,
}

enum Command {
    List,
    Inspect(TodoFieldDescribe),
    Add(CreateTodo),
    Set(TodoFieldDescribe, TodoFieldDescribe),
    Del(TodoFieldDescribe),
    Visual,
    Help,
}

const TABLE_TODO_NAME: &str = "iiran_todo";

const ENV_HOST: &str = "IIRAN_TODO_DB_HOST";
const ENV_PORT: &str = "IIRAN_TODO_DB_PORT";
const ENV_USER: &str = "IIRAN_TODO_DB_USER";
const ENV_PW: &str = "IIRAN_TODO_DB_PASSWORD";

const HELP_MAN: &str = "
tds 0.1.0
A utility that organize todo.

USAGE:
  tds [COMMAND] [OPTION]

COMMAND:
  l --list         List all todo status.
  i --inspect      Check todo.
  a --add          Create new todo.
  s --set          Update todo status.
  d --del          Delete todo.
  v --visual       Visual Mode
";

fn init_todo_table(c: &mut Client) -> Result<usize, String> {
    let q_res = c.simple_query(
        format!(
            "
       CREATE TABLE IF NOT EXISTS {}  (
           id            SERIAL         PRIMARY KEY     NOT NULL                   ,
           title         TEXT                           NOT NULL                   ,
           content       TEXT                           NOT NULL  DEFAULT   ''     ,
           author        VARCHAR(128)                   NOT NULL                   , 
           last_edit_by  VARCHAR(128)                   NOT NULL                   ,
           created_at    TIMESTAMP                      NOT NULL  DEFAULT   now()  ,
           updated_at    TIMESTAMP                      NOT NULL  DEFAULT   now()  ,
           deleted_at    TIMESTAMP                                DEFAULT   NULL   ,
           state         INT                            NOT NULL  DEFAULT   0      
       );
       CREATE INDEX  IF NOT EXISTS state_index  ON {}(state);
       ",
            TABLE_TODO_NAME, TABLE_TODO_NAME
        )
        .as_str(),
    );
    match &q_res {
        Ok(r) => Ok(r.len()),
        Err(r) => Err(r.to_string()),
    }
}

fn parse_raw_args() -> Command {
    let mut args: VecDeque<String> = env::args().skip(1).collect();
    match args.len() {
        0 => Command::List,
        i => {
            let rawcmd = args.pop_front().unwrap().to_lowercase().clone();
            let i = i - 1;
            match (rawcmd.as_str(), i) {
                ("l", 0) | ("--list", 0) => Command::List,
                ("a", 1) | ("--add", 1) => Command::Add(CreateTodo {
                    title: args[0].clone(),
                }),
                ("s", 3) | ("--set", 3) => Command::Set(UpdateTodo {
                    todo: args[0].clone(),
                    key: args[1].clone(),
                    value: args[2].clone(),
                }),
                ("d", 1) | ("--del", 1) => Command::Del(DeleteTodo {
                    todo: args[0].clone(),
                }),
                ("i", 1) | ("--inspect", 1) => Command::Inspect(InspectTodo {
                    todo: args[0].clone(),
                }),
                ("v", 0) | ("--visual", 0) => Command::Visual,
                ("h", 0) | ("--help", 0) => Command::Help,
                _ => Command::Help,
            }
        }
    }
}

fn main() {
    let cmd = parse_raw_args();

    dotenv().ok();
    let host = env::var(ENV_HOST).ok().unwrap_or_default();
    let user = env::var(ENV_USER).ok().unwrap_or_default();
    let password = env::var(ENV_PW).ok().unwrap_or_default();
    let port = env::var(ENV_PORT).ok().unwrap_or_default();
    let mut client = Client::connect(
        format!(
            "host={} port={} user={} password={}",
            host, port, user, password
        )
        .as_str(),
        NoTls,
    )
    .unwrap();

    init_todo_table(&mut client).unwrap();

    let cmd_res = match cmd {
        Command::Help => print_help(),
        Command::Add(add) => create_todo(&mut client, add),
        _ => print_help(),
    };

    if let Err(s) = cmd_res {
        println!("ERROR: {}", s)
    }
}

fn print_help() -> Result<(), String> {
    print!("{}", HELP_MAN);
    Ok(())
}

fn create_todo(c: &mut Client, todo: CreateTodo) -> Result<(), String> {
    let sql = format!(
        "INSERT INTO {} (title, author, last_edit_by) VALUES ($1, $2, $3)",
        TABLE_TODO_NAME
    );

    match c.execute(sql.as_str(), &[&todo.title, &todo.author, &todo.author]) {
        Ok(ra) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

fn list_todo(c: &mut Client) -> 