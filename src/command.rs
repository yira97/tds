use super::draw::sbui::*;
use super::remote::*;
use super::setting::Config;
use super::store::DB;
use super::time::{get_str_by_time, get_time_by_str};
use super::todo::*;
use chrono::{DateTime, Utc};
use colored::*;
use std::collections::VecDeque;
use std::env;

pub enum Command {
    InitStore,
    ReInitStore,
    List,
    Inspect(i32),
    Add(Todo),
    Set(Vec<i32>, ToDoState),
    Del(Vec<i32>),
    Visual,
    Help,
    Pull(RemoteToDoAPI),
}

impl Command {
    pub fn new_from_args() -> Self {
        let mut args: VecDeque<String> = env::args().skip(1).collect();
        match args.len() {
            0 => Command::List,
            i => {
                let rawcmd = args.pop_front().unwrap().to_lowercase().clone();
                let i = i - 1;
                match (rawcmd.as_str(), i) {
                    // example: --init
                    ("--init", _) => Command::InitStore,
                    // example: --reinit
                    ("--reinit", _) => Command::ReInitStore,
                    // example: l
                    ("l", 0) | ("--list", 0) => Command::List,
                    // example: a jog
                    ("a", 1) | ("--add", 1) => {
                        let mut td = Todo::new();
                        td.title = args[0].clone();
                        Command::Add(td)
                    }
                    // example: a jog 12h
                    ("a", 2) | ("--add", 2) => {
                        let due = get_time_by_str(args[1].as_str());
                        let mut td = Todo::new();
                        td.title = args[0].clone();
                        td.due_at = due;
                        Command::Add(td)
                    }
                    // example: s c 2
                    // example: s c 1 3 5 6 7
                    ("s", i) | ("--set", i) => {
                        if i == 0 {
                            return Command::Help;
                        }

                        let state = ToDoState::from(args[0].as_str());
                        if state == ToDoState::Unknown {
                            return Command::Help;
                        }
                        let mut vid = vec![];
                        for i in 1..i {
                            if let Ok(id) = get_n_from_charset(args[i].as_str()) {
                               vid.push(id);
                            }
                        }
                        if vid.len() >0 {
                            return Command::Set(vid,state);
                        }
                        Command::Help
                    }
                    // example: d 2
                    // example: d 1 3 5 6 7
                    ("d", i) | ("--del", i) => {
                        if i == 0 {
                            return Command::Help;
                        }
                        let mut vid = vec![];
                        for i in 0..i {
                            if let Ok(id) = get_n_from_charset(args[i].as_str()) {
                                vid.push(id);
                            }
                        }
                        if vid.len() > 0 {
                            return Command::Del(vid);
                        }
                        Command::Help
                    }
                    // example: i 3
                    ("i", 1) | ("--inspect", 1) => {
                        if let Ok(id) = get_n_from_charset(args[0].as_str()) {
                            return Command::Inspect(id);
                        }
                        Command::Help
                    }
                    // example: v
                    ("v", 0) | ("--visual", 0) => Command::Visual,
                    // example: h
                    ("h", 0) | ("--help", 0) => Command::Help,
                    // example: p gl
                    ("p", 1) | ("--pull", 1) => {
                        // second parameter select the api
                        // remote host were set in config
                        let api = RemoteToDoAPI::new(args[0].to_lowercase().as_str());
                        Command::Pull(api)
                    }
                    _ => Command::Help,
                }
            }
        }
    }

    pub fn run(&self, c: &mut DB, cfg: &Config) -> Result<(), &'static str> {
        match self {
            Command::InitStore => c.init_todo_table(),
            Command::ReInitStore => {
                if let Err(_) = c.drop_todo_table() {
                    println!("drop failed")
                }
                c.init_todo_table()
            }
            Command::List => match c.get_todos() {
                Ok(tds) => list_todo(tds),
                Err(_) => Err("list todo failed"),
            },
            Command::Inspect(target) => match c.get_todo(*target) {
                Ok(td) => print_todo_detail(td),
                Err(_) => Err("print todo detail failed"),
            },
            Command::Add(td) => match c.create_todo(td) {
                Ok(()) => Ok(()),
                Err(_) => Err("create todo failed"),
            },
            Command::Set(targets, state) => {
                let mut verr = vec![];
                for target in targets {
                    if let Err(e) = c.update_todo_state(*target, *state) {
                        verr.push(e);
                    }
                }
                match verr.len() {
                    0 => Ok(()),
                    _i => Err("fail"),
                }
            },
            Command::Del(targets) => {
                let mut verr = vec![];
                for target in targets {
                    if let Err(e) = c.delete_todo(*target) {
                        verr.push(e);
                    }
                }
                match verr.len() {
                    0 => Ok(()),
                    _i => Err("delete toto failed"),
                }
            },
            Command::Visual => enter_visual(),
            Command::Help => print_help(),
            //TODO: ugly
            Command::Pull(_api) => {
                let gl = GitLabOrigin::new(cfg.gitlab_domain.clone(), cfg.gitlab_ac_token.clone());
                match gl.pull() {
                    Err(_) => Err("pull failed"),
                    Ok(vtd) => {
                        for td in vtd.iter() {
                            match c.create_todo(&td) {
                                Ok(_) => (),
                                Err(_) => (),
                            }
                        }
                        Ok(())
                    }
                }
            }
        }
    }
    pub fn is_write_cmd(&self) -> bool {
        match self {
            // ignore initstore
            Command::Add(_) | Command::Del(_) | Command::Set(_, _) => true,
            _ => false,
        }
    }
}

fn enter_visual() -> Result<(), &'static str> {
    println!("enter visual mode");
    Ok(())
}

fn print_todo_detail(td: Todo) -> Result<(), &'static str> {
    println!("{:?}", td);
    Ok(())
}

fn print_help() -> Result<(), &'static str> {
    const HELP_MAN: &str = r#"
    tds 0.1.1
    A tool to manage to-do items.

    USAGE:
    tds [COMMAND] [OPTION]

    COMMAND:
    l --list,                           List all todo status.
    i --inspect <ID>                    Check todo.
    a --add  <title> <due>              Create new todo.
    s --set <state>  <ID>...            Update todo status.
    d --del <ID>...                     Delete todo.
    v --visual                          Visual Mode.
    p --pull  <host>                    Pull ToDo from gitlab / github.
    "#;
    println!("{}", HELP_MAN);
    Ok(())
}

fn list_todo(tds: Vec<Todo>) -> Result<(), &'static str> {
    let now = Utc::now();

    let mut next = Vec::new();
    let mut inprog = Vec::new();
    let mut review = Vec::new();
    let mut complete = Vec::new();
    let mut closed = Vec::new();

    for td in tds.iter() {
        let tdd = TodoDisplay {
            id: get_c_from_charset(td.id),
            title: td.title.clone(),
            due: get_todo_due_str(td.due_at, now),
            run: get_str_by_time(td.created_at, now),
            state: td.state,
        };

        match tdd.state {
            ToDoState::NextUp => next.push(tdd),
            ToDoState::InProgress => inprog.push(tdd),
            ToDoState::Review => review.push(tdd),
            ToDoState::Completed => complete.push(tdd),
            ToDoState::Closed => closed.push(tdd),
            _ => (),
        }
    }

    let mut win = SymbolWindow::new();
    win.add_tag(&["TASK", "DUE", "RUN", "ID"]);
    win.change_weight("TASK", |w| 3.0 * w);
    win.change_weight("DUE", |w| 0.4 * w);
    win.change_weight("RUN", |w| 0.4 * w);
    win.change_weight("ID", |w| 0.3 * w);
    win.refresh();
    print_title!(win);
    print_div!(win, format!("{}({})", ToDoState::NextUp, next.len()), green);
    print_rows!(win,next, &[title; due; run; id]);
    print_div!(
        win,
        format!("{}({})", ToDoState::InProgress, inprog.len()),
        yellow
    );
    print_rows!(win,inprog, &[title; due; run; id]);
    print_div!(
        win,
        format!("{}({})", ToDoState::Review, review.len()),
        purple
    );
    print_rows!(win,review, &[title; due; run; id]);
    print_foot!(win);
    Ok(())
}

fn get_todo_due_str(due: Option<DateTime<Utc>>, now: DateTime<Utc>) -> String {
    match due {
        Some(due_at) => {
            if due_at < now {
                String::from("TIMEOUT")
            } else {
                get_str_by_time(due_at, now)
            }
        }
        None => String::from("-"),
    }
}

const CHAR_SET: &[u8] = "0123456789abcdefghijklmnopqrstuvwxyz".as_bytes();
const CHAR_SET_LEN: i32 = CHAR_SET.len() as i32;

fn get_c_from_charset(i: i32) -> String {
    debug_assert!(i >= 0);

    let mut vc = VecDeque::new();
    let mut remain = i;

    loop {
        let c_i = (remain % CHAR_SET_LEN) as usize;
        vc.push_front(CHAR_SET[c_i] as char);
        remain /= CHAR_SET_LEN;
        if remain == 0 {
            break
        }
    }
    vc.iter().collect()
}

#[test]
fn test_charset_conv() {
    assert_eq!(get_c_from_charset(0), "0");
    assert_eq!(get_c_from_charset(9), "9");
    assert_eq!(get_c_from_charset(10), "a");
    assert_eq!(get_c_from_charset(35), "z");
    assert_eq!(get_c_from_charset(36), "10");

    assert_eq!(get_n_from_charset("0"),Ok(0));
    assert_eq!(get_n_from_charset("9"),Ok(9));
    assert_eq!(get_n_from_charset("a"),Ok(10));
    assert_eq!(get_n_from_charset("z"),Ok(35));
    assert_eq!(get_n_from_charset("10"),Ok(36));

    assert_eq!(get_n_from_charset("10F"),Err(()));
}

fn get_n_from_charset(s: &str) -> Result<i32,()> {
    let mut sum:i32 = 0;
    for c in s.as_bytes().iter() {
        match CHAR_SET.iter().position(|c_in_set|c_in_set == c) {
            Some(i) => {
                sum += i as i32;
                sum *= CHAR_SET_LEN;
            }
            None => return Err(()),
        }
    }
    sum = sum / CHAR_SET_LEN;
    Ok(sum)
}
