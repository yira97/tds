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

fn get_db_info_from_env() -> (String, String, String, String) {
    const ENV_HOST: &str = "TDS_DB_HOST";
    const ENV_PORT: &str = "TDS_DB_PORT";
    const ENV_USER: &str = "TDS_DB_USER";
    const ENV_PW: &str = "TDS_DB_PASSWORD";
    const ENV_DB: &str = "TDS_DB_DB";
    let host = env::var(ENV_HOST).ok().unwrap_or_default();
    let port = env::var(ENV_PORT).ok().unwrap_or_default();
    let user = env::var(ENV_USER).ok().unwrap_or_default();
    let password = env::var(ENV_PW).ok().unwrap_or_default();
    (host, port, user, password)
}
