# tds

[![Latest Version](https://img.shields.io/crates/v/tds.svg)](https://crates.io/crates/tds)

A tool to manage to-do items.

![banner](.readme/banner.png)

## Install

### 0. vi ~/.config/tds/tds.toml

```toml
db_host = "127.0.0.1"
db_port = "5432"
db_user = "iiran"
db_password = "iiran"
db_database = "iiran"
user = "iiran"
email = "percivalstr@163.com"
gitlab_user = ""
gitlab_ac_token = ""
```

### 1. Install by Cargo

```bash
cargo install tds
tds --init
```

## How to use

```bash
    l --list         List all todo status.
    i --inspect      Check todo.
    a --add          Create new todo.
    s --set          Update todo status.
    d --del          Delete todo.
    v --visual       Visual Mode.
    p --pull         pull todo from gitlab
```
