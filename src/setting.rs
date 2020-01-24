use serde::{Deserialize, Serialize};
use std::env;
use std::fs;

#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    pub db_host: String,
    pub db_port: String,
    pub db_user: String,
    pub db_password: String,
    pub db_database: String,
    pub user: String,
    pub email: String,
    pub gitlab_user: String,
    pub gitlab_ac_token: String,
    pub gitlab_domain: String,
    pub show_run_time: bool,
    pub show_due_time: bool,
    pub show_content: bool,
    pub order_by: String,
    pub task_title_align: String,
    pub window_width: i32,
    pub use_gitlab_todo_crate_time: bool,
    pub show_num_on_div: bool,
}

const CONFIG_DIR: &str = ".config/tds/";
const CONFIG_FILENAME: &str = "tds.toml";
const DEFAULT_CONFIG: &str = r#"
db_host = "127.0.0.1"
db_port = "5432"
db_user = "yiranfeng"
db_password = "fyr"
db_database = "yiranfeng"
user = "iiran"
email = "percivalstr@163.com"
gitlab_user = ""
gitlab_ac_token = ""
gitlab_domain = "gitlab.com"
use_gitlab_todo_crate_time = false
show_run_time = true
show_due_time = true
show_content = false
order_by = "run"
task_title_align = "mid"
window_width = 80
show_num_on_div = true
"#;

pub fn init_config() -> Config {
    // TODO: use $XDG_CONFIG_HOME as default
    let home = match dirs::home_dir() {
        Some(d) => d,
        None => match env::current_dir() {
            Ok(d) => d,
            Err(e) => panic!("??? {}", e),
        },
    };
    let p = home.join(CONFIG_DIR);
    let cfg = p.join(CONFIG_FILENAME);
    if !p.exists() {
        // create and initialize
        if let Err(e) = fs::create_dir_all(&p) {
            panic!("can not create dir. {}", e);
        }
        if let Err(e) = fs::write(&cfg, DEFAULT_CONFIG) {
            panic!("config initialize error. {}", e);
        }
    }
    let cfg_data = match fs::read(&cfg) {
        Ok(data) => data,
        Err(e) => panic!("failed to read config. {}", e),
    };

    let cfg = match toml::from_slice::<Config>(&cfg_data[..]) {
        Err(e) => panic!("fail when parse the config. {}", e),
        Ok(v) => v,
    };
    cfg
}

#[allow(dead_code)] // useful
fn get_config_from_env() -> Config {
    const ENV_HOST: &str = "TDS_DB_HOST";
    const ENV_PORT: &str = "TDS_DB_PORT";
    const ENV_USER: &str = "TDS_DB_USER";
    const ENV_PW: &str = "TDS_DB_PASSWORD";
    const ENV_DB: &str = "TDS_DB_DB";
    
    let host = env::var(ENV_HOST).ok().unwrap_or_default();
    let port = env::var(ENV_PORT).ok().unwrap_or_default();
    let user = env::var(ENV_USER).ok().unwrap_or_default();
    let password = env::var(ENV_PW).ok().unwrap_or_default();
    let database = env::var(ENV_DB).ok().unwrap_or_default();

    Config{
        db_host :host,
        db_port: port,
        db_user: user,
        db_password: password,
        db_database: database,
        user:String::new(),
        email: String::new(),
        gitlab_user:String::new(),
        gitlab_ac_token:String::new(),
        gitlab_domain:String::new(),
        show_run_time:true,
        show_due_time:true,
        show_content:false,
        order_by:String::from("run"),
        task_title_align:String::from("mid"),
        window_width: 80,
        use_gitlab_todo_crate_time: false,
        show_num_on_div: true,
    }
}
