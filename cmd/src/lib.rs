use clap::{ arg, command, value_parser };
use std::path::PathBuf;
use colored::Colorize;


pub fn main() {
    println!("{}", r"
   __    _   __   __    __            _____   ___    _  __
  / /   (_) / /_ / /_  / / ___       / ___/  / _ \  / |/ /
 / /__ / / / __// __/ / / / -_)     / /__   / // / /    /
/____//_/  \__/ \__/ /_/  \__/      \___/  /____/ /_/|_/
    ".red().bold());

    let matches = command!()
        .arg(arg!(
            -s --"stop-web" "Don't run the web view"
        ))
        .arg(arg!(
                -a --address <ADDRESS> "Address on which server should run"
            )
            .default_value("127.0.0.1")
        )
        .arg(arg!(
                -p --port <PORT> "Port for the server"
            )
            .default_value("3000")
        )
        .arg(arg!(
            -l --log "Show api logs"
        ))
        .arg(arg!(
                -d --dir <DIR> "Main folder for storing files"
            )
            .value_parser(value_parser!(PathBuf))
            .default_value("files")
        )
        .arg(arg!(
                --db <DATABASE> "Database folder name"
            )
            .value_parser(value_parser!(PathBuf))
            .default_value("db")
        )
        .arg(arg!(
            --"clear-database" "Clear the database"
        ))
        .arg(arg!(
            --"disable-login" --"d-l" "No one will be allowed to create new account"
        ))
        .get_matches();

    let stop_web = matches.get_one::<bool>("stop-web").unwrap().clone();
    let address = matches.get_one::<String>("address").unwrap().clone();
    let port = matches.get_one::<String>("port").unwrap().clone().parse::<u16>().unwrap();
    let log= matches.get_one::<bool>("log").unwrap().clone();
    let dir = matches.get_one::<PathBuf>("dir").unwrap().clone();
    let db = matches.get_one::<PathBuf>("db").unwrap().clone();
    let disable_login = matches.get_one::<bool>("disable-login").unwrap().clone();
    let clear_database = matches.get_one::<bool>("clear-database").unwrap().clone();

    let server = little_cdn_api::main(
        stop_web,
        address,
        port,
        log,
        dir,
        db,
        clear_database,
        disable_login
    );

    match server {
        Ok(v) => v,
        Err(e) => panic!("{}", e),
    }
}