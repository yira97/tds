use super::setting::Config;
use super::todo::{ToDoState, Todo, TodoRef};
use chrono::{DateTime, Utc};
use postgres::{Client, NoTls};

const TABLE_TODO_NAME: &str = "iiran_todo";
const TABLE_TODO_REF: &str = "iiran_todo_ref";

pub struct DB {
    conn: Option<Client>,
    user: String,
    pub user_gitlab_username: String,
    pub user_gitlab_domain: String,
}

impl DB {
    pub fn new(cfg: &Config) -> Self {
        let mut db = DB {
            conn: None,
            user: cfg.user.clone(),
            user_gitlab_domain: cfg.gitlab_domain.clone(),
            user_gitlab_username: cfg.gitlab_user.clone(),
        };
        db.conn = match Client::connect(
            format!(
                "host={} port={} user={} password={}",
                cfg.db_host, cfg.db_port, cfg.db_user, cfg.db_password
            )
            .as_str(),
            NoTls,
        ) {
            Ok(c) => Some(c),
            Err(_) => None,
        };
        db
    }

    pub fn update_todo_state(&mut self, id: i32, state: ToDoState) -> Result<(), String> {
        let now = Utc::now();
        let sql = format!(
            r#"
        UPDATE {0} 
        SET state = $1, last_edit_by = $2, updated_at = $3 
        WHERE id = $4;
        "#,
            TABLE_TODO_NAME
        );
        let q_res = self
            .conn
            .as_mut()
            .unwrap()
            .execute(sql.as_str(), &[&(state as i32), &self.user, &now, &id]);
        match q_res {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    }

    pub fn delete_todo(&mut self, id: i32) -> Result<(), String> {
        let sql = format!(
            r#"
        UPDATE {0}
        SET deleted_at = $1, last_edit_by = $2, updated_at = $3
        WHERE id = $4
        "#,
            TABLE_TODO_NAME
        );
        let now = Utc::now();
        let q_res = self
            .conn
            .as_mut()
            .unwrap()
            .execute(sql.as_str(), &[&now, &self.user, &now, &id]);
        match q_res {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    }

    pub fn drop_todo_table(&mut self) -> Result<(), &'static str> {
        let q_res = self.conn.as_mut().unwrap().simple_query(
            format!(
                r#"
                DROP TABLE IF EXISTS  {table_todo};
                DROP TABLE IF EXISTS  {table_todo_ref};
                "#,
                table_todo = TABLE_TODO_NAME,
                table_todo_ref = TABLE_TODO_REF
            )
            .as_str(),
        );
        match q_res {
            Ok(_) => Ok(()),
            Err(_e) => Err("drop table error"),
        }
    }

    pub fn init_todo_table(&mut self) -> Result<(), &'static str> {
        let q_res = self.conn.as_mut().unwrap().simple_query(
            format!(
            r#"
            CREATE TABLE IF NOT EXISTS {table_todo} (
                id            SERIAL         PRIMARY KEY     NOT NULL                                    ,
                title         TEXT                           NOT NULL                                    ,
                content       TEXT                           NOT NULL  DEFAULT   ''                      ,
                author        VARCHAR(128)                   NOT NULL  DEFAULT   ''                      ,
                assignee      VARCHAR(128)                   NOT NULL  DEFAULT   ''                      ,
                last_edit_by  VARCHAR(128)                   NOT NULL  DEFAULT   ''                      ,
                created_at    TIMESTAMPTZ                    NOT NULL  DEFAULT   now()                   ,
                updated_at    TIMESTAMPTZ                    NOT NULL  DEFAULT   now()                   ,
                deleted_at    TIMESTAMPTZ                              DEFAULT   NULL                    ,
                due_at        TIMESTAMPTZ                              DEFAULT   NULL                    ,
                state         INT                            NOT NULL  DEFAULT   0                       ,
                weight        INT                            NOT NULL  DEFAULT   1                       ,  
                premise       INT                                      DEFAULT   NULL                    ,
                rf_id         INT                                      DEFAULT   NULL                     
            );

            CREATE INDEX  IF NOT EXISTS state_index  ON {table_todo}(state);
            CREATE INDEX  IF NOT EXISTS due_index    ON {table_todo}(due_at);

            CREATE TABLE IF NOT EXISTS {table_todo_ref} (
                id                 SERIAL                PRIMARY KEY              NOT NULL               ,
                domain             VARCHAR(512)                                   NOT NULL               ,
                username           VARCHAR(256)                                   NOT NULL               ,
                todo_id            VARCHAR(64)                                    NOT NULL               ,
                project_id         VARCHAR(64)                                    NOT NULL               ,
                project_name       VARCHAR(512)                                   NOT NULL               ,
                author_username    VARCHAR(256)                        DEFAULT    NULL                   ,
                action_name        VARCHAR(256)                        DEFAULT    NULL                   ,
                target_type        VARCHAR(64)                         DEFAULT    NULL                   ,
                created_at         TIMESTAMPTZ                         DEFAULT    NULL                  
            );

            CREATE UNIQUE INDEX IF NOT EXISTS unique_todo_id_index ON {table_todo_ref}(domain, username, todo_id);
            "#,
            table_todo = TABLE_TODO_NAME,
            table_todo_ref = TABLE_TODO_REF,
            )
            .as_str(),
        );
        match &q_res {
            Ok(_) => Ok(()),
            Err(_r) => Err("init todo table failed"),
        }
    }

    pub fn get_todo(&mut self, id: i32) -> Result<Todo, &'static str> {
        let sql = format!(
            r#"
        SELECT id, title, content, author, last_edit_by, created_at, updated_at, state, weight, premise, due_at, rf_id
        FROM {0}
        WHERE deleted_at IS NULL AND ID = $1 AND assignee = $2
        LIMIT 1
        "#,
            TABLE_TODO_NAME
        );
        match self
            .conn
            .as_mut()
            .unwrap()
            .query(sql.as_str(), &[&id, &self.user])
        {
            Ok(rows) => match rows.first() {
                Some(row) => {
                    let id: i32 = row.get(0);
                    let title: String = row.get(1);
                    let content: String = row.get(2);
                    let author: String = row.get(3);
                    let last_edit_by: String = row.get(4);
                    let created_at: DateTime<Utc> = row.get(5);
                    let updated_at: DateTime<Utc> = row.get(6);
                    let state: i32 = row.get(7);
                    let _weight: i32 = row.get(8);
                    let _premise: i32 = row.get(9);
                    let due_at: Option<DateTime<Utc>> = row.get(10);
                    let state = ToDoState::from(state);
                    let rf_id: Option<i32> = row.get(11);
                    let rf = match rf_id {
                        Some(rf_id) => self.get_todo_rf(rf_id),
                        None => None,
                    };
                    let td = Todo {
                        id: id,
                        title: title,
                        content: content,
                        author: author,
                        assignee: self.user.clone(),
                        last_edit_by: last_edit_by,
                        created_at: created_at,
                        updated_at: updated_at,
                        due_at: due_at,
                        state: state,
                        rf: rf,
                    };
                    Ok(td)
                }
                None => Err("no result"),
            },
            Err(_) => Err("select todo error"),
        }
    }

    //// this is private method
    //// used by `get_todo`
    fn get_todo_rf(&mut self, rf_id: i32) -> Option<TodoRef> {
        let sql = format!(
            r#"
        SELECT domain, todo_id, project_id, project_name, author_username, action_name, target_type, created_at
        FROM {table_todo_ref}
        WHERE id = $1
        LIMIT 1
        "#,
            table_todo_ref = TABLE_TODO_REF
        );

        let rf = match self.conn.as_mut().unwrap().query(sql.as_str(), &[&rf_id]) {
            Ok(rows) => match rows.first() {
                Some(row) => {
                    let mut rf = TodoRef::new();
                    rf.domain = row.get(0);
                    rf.todo_id = row.get(1);
                    rf.project_id = row.get(2);
                    rf.project_name = row.get(3);
                    rf.author_username = row.get(4);
                    rf.action_name = row.get(5);
                    rf.target_type = row.get(6);
                    rf.created_at = row.get(7);
                    Some(rf)
                }
                None => None,
            },
            Err(_) => None,
        };
        rf
    }

    //// # remark
    //// fetch all todo which assignee point to login user (by config)
    pub fn get_todos(&mut self) -> Result<Vec<Todo>, &'static str> {
        let sql = format!(
            r#"
            SELECT id, title, content, author, last_edit_by, created_at, updated_at, due_at, state, rf_id
            FROM {} 
            WHERE deleted_at IS NULL AND assignee = $1
            ORDER BY id ASC;
        "#,
            TABLE_TODO_NAME
        );
        match self
            .conn
            .as_mut()
            .unwrap()
            .query(sql.as_str(), &[&self.user])
        {
            Ok(rows) => {
                let mut tds = vec![];
                for row in rows.iter() {
                    let id: i32 = row.get(0);
                    let title: String = row.get(1);
                    let content: String = row.get(2);
                    let author: String = row.get(3);
                    let last_edit_by: String = row.get(4);
                    let created_at: DateTime<Utc> = row.get(5);
                    let updated_at: DateTime<Utc> = row.get(6);
                    let due_at: Option<DateTime<Utc>> = row.get(7);
                    let state: i32 = row.get(8);
                    let state = ToDoState::from(state);
                    let rf_id: Option<i32> = row.get(9);
                    let rf = match rf_id {
                        Some(rf_id) => self.get_todo_rf(rf_id),
                        None => None,
                    };
                    let td = Todo {
                        id: id,
                        title: title,
                        content: content,
                        author: author,
                        assignee: self.user.clone(),
                        last_edit_by: last_edit_by,
                        created_at: created_at,
                        updated_at: updated_at,
                        due_at: due_at,
                        state: state,
                        rf: rf,
                    };
                    tds.push(td);
                }
                Ok(tds)
            }
            Err(_) => Err("select todo error"),
        }
    }

    //// return rf_id
    //// # remark
    //// the domain and username field in ToDoRef will be ignored.
    fn create_todo_ref(&mut self, rf: &TodoRef) -> Result<i32, &'static str> {
        let sql = format!(
            r#"
            INSERT INTO {table_todo_ref} (domain, username, todo_id, project_id, project_name, author_username, action_name, target_type, created_at)
            VALUES
            ($1, $2, $3, $4, $5, $6, $7, $8, $9);
            "#,
            table_todo_ref = TABLE_TODO_REF
        );
        // because self will happend mut borrow later, so here we must clone.
        let default_domain = self.user_gitlab_domain.clone();
        let default_username = self.user_gitlab_username.clone();
        match self.conn.as_mut().unwrap().execute(
            sql.as_str(),
            &[
                &default_domain,
                &default_username,
                &rf.todo_id,
                &rf.project_id,
                &rf.project_name,
                &rf.author_username,
                &rf.action_name,
                &rf.target_type,
                &rf.created_at,
            ],
        ) {
            Ok(_) => match self.get_todo_ref_id(&default_domain, &default_username, &rf.todo_id) {
                Some(id) => Ok(id),
                None => Err("get id failed"),
            },
            Err(_) => Err("insert failed"),
        }
    }

    fn get_todo_ref_id(&mut self, domain: &str, username: &str, todo_id: &str) -> Option<i32> {
        let sql = format!(
            r#"
            SELECT id 
            FROM {table_todo_ref} 
            WHERE domain = $1 AND username = $2 AND todo_id = $3
            LIMIT 1
            "#,
            table_todo_ref = TABLE_TODO_REF
        );

        match self
            .conn
            .as_mut()
            .unwrap()
            .query(sql.as_str(), &[&domain, &username, &todo_id])
        {
            Err(_) => None,
            Ok(rows) => match rows.first() {
                None => None,
                Some(row) => {
                    let id: i32 = row.get(0);
                    Some(id)
                }
            },
        }
    }

    //// # remark
    //// author and assignee will be set to login user (by config) automaticly,
    //// if you leave anything on these field, will surpass the default behavor.
    pub fn create_todo(&mut self, todo: &Todo) -> Result<(), &'static str> {
        let sql = format!(
            "INSERT INTO {table_todo_name} (title, author, assignee, last_edit_by, due_at, rf_id) VALUES ($1, $2, $3, $4, $5, $6)",
            table_todo_name=TABLE_TODO_NAME
        );
        // because self will happends mut borrow, here we must clone.
        let default_user = self.user.clone();
        //  default author is login user, if todo.author is ""
        let author = match todo.author.as_str() {
            "" => &default_user,
            _ => &todo.author,
        };
        // default assignee is login user, if todo.assignee is ""
        let assignee = match todo.assignee.as_str() {
            "" => &default_user,
            _ => &todo.assignee,
        };

        let rf_id: Option<i32> = match &todo.rf {
            None => None,
            Some(rf) => match self.create_todo_ref(rf) {
                Err(_) => return Err("already exist"),
                Ok(id) => Some(id),
            },
        };

        // last_edit_by is author
        match self.conn.as_mut().unwrap().execute(
            sql.as_str(),
            &[&todo.title, author, assignee, &author, &todo.due_at, &rf_id],
        ) {
            Ok(_) => Ok(()),
            Err(_) => Err("create todo failed"),
        }
    }
}
