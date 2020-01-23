#[macro_use]
mod draw;
mod command;
mod remote;
mod setting;
mod store;
mod time;
mod todo;

fn main() {
    let cfg = setting::init_config();
    let cmd = command::Command::new_from_args();
    let mut db = store::DB::new(&cfg);
    if let Err(e) = cmd.run(&mut db, &cfg) {
        println!("ERROR: {}", e)
    } else if cmd.is_write_cmd() {
        command::Command::List.run(&mut db, &cfg).unwrap();
        ()
    }
}
