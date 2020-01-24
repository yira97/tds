use chrono::{DateTime, Utc};

//// TODO STATE BEG <<<<<<<<<<<<<<<<<<<

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ToDoState {
    NextUp,
    InProgress,
    Review,
    Completed,
    Closed,
    Unknown,
}

impl From<i32> for ToDoState {
    fn from(n: i32) -> Self {
        match n {
            0 => ToDoState::NextUp,
            1 => ToDoState::InProgress,
            2 => ToDoState::Review,
            3 => ToDoState::Completed,
            4 => ToDoState::Closed,
            _ => ToDoState::Unknown,
        }
    }
}

impl From<&str> for ToDoState {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "n" | "next" => ToDoState::NextUp,
            "p" | "progress" => ToDoState::InProgress,
            "r" | "review" => ToDoState::Review,
            "c" | "cp" | "complete" | "completed" => ToDoState::Completed,
            "cl" | "close" | "closed" => ToDoState::Closed,
            _ => ToDoState::Unknown,
        }
    }
}

impl Into<Option<i32>> for ToDoState {
    fn into(self) -> Option<i32> {
        match self {
            ToDoState::NextUp => Some(0),
            ToDoState::InProgress => Some(1),
            ToDoState::Review => Some(2),
            ToDoState::Completed => Some(3),
            ToDoState::Closed => Some(4),
            ToDoState::Unknown => None,
        }
    }
}

impl std::fmt::Display for ToDoState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ToDoState::NextUp => "Next Up",
            ToDoState::InProgress => "In Progress",
            ToDoState::Review => "Review",
            ToDoState::Completed => "Completed",
            ToDoState::Closed => "Closed",
            _ => "Unknown",
        };
        write!(f, "{}", s)
    }
}

//// TODO STATE END >>>>>>>>>>>>
///
/// TODO REF BEG <<<<<<<<<<<<<<<

#[derive(Debug)]
pub struct TodoRef {
    pub domain: String,
    pub username: String,
    pub todo_id: String,
    pub project_id: String,
    pub project_name: String,
    pub author_username: Option<String>,
    pub target_type: Option<String>,
    pub action_name: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
}

impl TodoRef {
    pub fn new() -> Self {
        TodoRef {
            domain: String::new(),
            username: String::new(),
            todo_id: String::new(),
            project_id: String::new(),
            project_name: String::new(),
            author_username: None,
            action_name: None,
            target_type: None,
            created_at: None,
        }
    }
}

//// TODO REF END >>>>>>>>>>>>>>>>>>>>>

#[derive(Debug)]
pub struct Todo {
    pub id: i32,
    pub title: String,
    pub content: String,
    pub author: String,
    pub assignee: String,
    pub last_edit_by: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub due_at: Option<DateTime<Utc>>,
    pub state: ToDoState,
    pub rf: Option<TodoRef>,
}

impl Todo {
    pub fn new() -> Self {
        let now = Utc::now();
        Todo {
            id: 0,
            title: String::new(),
            content: String::new(),
            author: String::new(),
            assignee: String::new(),
            last_edit_by: String::new(),
            created_at: now,
            updated_at: now,
            due_at: None,
            state: ToDoState::NextUp,
            rf: None,
        }
    }
}

#[derive(Debug)]
pub struct TodoDisplay {
    pub id: String,
    pub title: String,
    pub due: String,
    pub run: String,
    pub state: ToDoState,
}
