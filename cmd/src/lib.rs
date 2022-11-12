use std::fs::create_dir;
use clap::{ arg, command, value_parser };
use std::path::PathBuf;
use colored::Colorize;

use little_cdn_api::Config;

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
            -l --log ... "Show api logs. Use multiple times for highier debuging level "
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
            -f --"clear-database" "Clear the database"
        ))
        .arg(arg!(
            -n --"disable-login" "No one will be allowed to create new account"
        ))
        .get_matches();

    let config = Config {
        stop_web: matches.get_one::<bool>("stop-web").unwrap().clone(),
        address: matches.get_one::<String>("address").unwrap().clone(),
        port: matches.get_one::<String>("port").unwrap().clone().parse::<u16>().unwrap(),
        log: matches.get_one::<u8>("log").unwrap().clone(),
        dir: matches.get_one::<PathBuf>("dir").unwrap().clone(),
        db: matches.get_one::<PathBuf>("db").unwrap().clone(),
        disable_login: matches.get_one::<bool>("disable-login").unwrap().clone(),
        clear_database: matches.get_one::<bool>("clear-database").unwrap().clone(),
    };

    let style_dir = create_dir("stylesheets/");
    match style_dir {
        Ok(v) => v,
        Err(_) => {
            if config.log > 1 {
                println!("Styles folder already exists skipping...");
            }
        },
    };
    let html_dir = create_dir("templates/");
    match html_dir {
        Ok(v) => v,
        Err(_) => {
            if config.log > 1 {
                println!("Templates folder already exists skipping...");
            }
        },
    };

    let server = little_cdn_api::main(config);

    match server {
        Ok(v) => v,
        Err(e) => panic!("{}", e),
    }
}
