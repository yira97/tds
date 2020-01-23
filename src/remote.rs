use super::todo::{ToDoState, Todo, TodoRef};
use chrono::DateTime;

pub enum RemoteToDoAPI {
    GitLab,
    GitHub,
}

impl RemoteToDoAPI {
    //// gitlab as default api when parse failed.
    pub fn new(api: &str) -> Self {
        match api {
            "gitlab" | "gl" | "l" => RemoteToDoAPI::GitLab,
            "github" | "gh" | "h" => RemoteToDoAPI::GitHub,
            _ => RemoteToDoAPI::GitLab,
        }
    }
}

pub struct GitLabOrigin {
    pub domain: String,
    pub access_token: String,
}

pub struct GitHubOrigin {}

impl OriginProtocol for GitHubOrigin {
    fn pull(&self) -> Result<Vec<Todo>, &'static str> {
        Err("not implement")
    }
}

impl GitLabOrigin {
    pub fn new(domain: String, access_token: String) -> Self {
        GitLabOrigin {
            domain,
            access_token,
        }
    }
}

impl OriginProtocol for GitLabOrigin {
    // id: number                                                     - > ref            v
    // project.id number                                              - > ref            v
    // project.path_with_namesapce string                             - > ref            v
    // author.username string                                         - > ref            v
    // action_name: string                                            - > ref            v
    // target_type: string                                            - > ref            v
    // body String                                                    - > todo.title     v
    // state string                                                   - > todo.state     v
    // created_at string (format: "2018-12-17T07:42:44.347Z")         - > ref            v

    fn pull(&self) -> Result<Vec<Todo>, &'static str> {
        let github_todo_get = format!("https://{}/api/v4/todos", self.domain.as_str());
        let client = reqwest::blocking::Client::new();
        let mut tds = vec![];

        let req_res = match client
            .get(github_todo_get.as_str())
            .header("PRIVATE-TOKEN", self.access_token.as_str())
            .send()
        {
            Err(_) => return Err("request error"),
            Ok(v) => v,
        };

        let parse_res = match req_res.json::<serde_json::Value>() {
            Err(_) => return Err("parse body error"),
            Ok(v) => v,
        };

        let raws = match parse_res.as_array() {
            None => return Err("expect array"),
            Some(raws) => raws,
        };

        for raw in raws.iter() {
            let mut td = Todo::new();
            let mut rf = TodoRef::new();
            rf.domain = self.domain.clone();
            match raw.get("body") {
                None => return Err("expect title"),
                Some(title) => match title.as_str() {
                    Some(s) => td.title = s.to_string(),
                    None => return Err("expect string  format title"),
                },
            }

            match raw.get("created_at") {
                None => return Err("expect created time"),
                Some(created_at) => match created_at.as_str() {
                    Some(s) => match DateTime::parse_from_rfc3339(s) {
                        Ok(t) => rf.created_at = Some(DateTime::from(t)),
                        Err(_) => return Err("expect rfc3339 format time"),
                    },
                    None => return Err("expect string create time"),
                },
            }
            match raw.get("id") {
                None => return Err("expect id"),
                Some(id) => match id.as_i64() {
                    Some(n) => rf.todo_id = n.to_string(),
                    None => return Err("expect number format id"),
                },
            }

            match raw.get("project") {
                None => return Err("expect project"),
                Some(obj_project) => {
                    match obj_project.get("id") {
                        None => return Err("expect project id"),
                        Some(id) => match id.as_i64() {
                            Some(n) => rf.project_id = n.to_string(),
                            None => return Err("expect i64 format project id"),
                        },
                    }

                    match obj_project.get("path_with_namespace") {
                        None => return Err("expect path with namepsace"),
                        Some(path) => match path.as_str() {
                            Some(path) => rf.project_name = path.to_string(),
                            None => return Err("expect string format path"),
                        },
                    }
                }
            }

            match raw.get("author") {
                Some(obj_author) => match obj_author.get("username") {
                    Some(username) => match username.as_str() {
                        Some(username) => rf.author_username = Some(username.to_string()),
                        None => return Err("expect string fromat username"),
                    },
                    None => return Err("expect username"),
                },
                None => return Err("expect author {}"),
            }

            match raw.get("state") {
                Some(state) => {
                    let state = match state.as_str() {
                        Some(state) => match state {
                            "pending" => "next",
                            _ => state,
                        },
                        None => return Err("expect string format state"),
                    };
                    td.state = ToDoState::from(state);
                }
                None => return Err("expect state"),
            }

            match raw.get("target_type") {
                None => return Err("expect target_type"),
                Some(target_type) => match target_type.as_str() {
                    Some(target_type) => {
                        rf.target_type = Some(target_type.to_string());
                    }
                    None => return Err("expect string format target_type"),
                },
            }

            match raw.get("action_name") {
                None => return Err("expect action_name"),
                Some(action_name) => match action_name.as_str() {
                    Some(action_name) => rf.action_name = Some(action_name.to_string()),
                    None => return Err("expect string format action_name"),
                },
            }
            td.rf = Some(rf);
            tds.push(td);
        }
        Ok(tds)
    }
}

pub trait OriginProtocol {
    fn pull(&self) -> Result<Vec<Todo>, &'static str>;
}
