use clap::{arg, command, value_parser};
use lib::{Config, LogLevel, error::AppError};
use std::{path::PathBuf, net::Ipv4Addr, fs};
use colored::Colorize;

pub fn init() -> Result<Config, AppError> {
    let matches = command!()
        .arg(arg!(-s --"stop-web" "Don't run the web view"))
        .arg(arg!(-a --address <ADDRESS> "Address on which server should run").value_parser(value_parser!(Ipv4Addr)).default_value("127.0.0.1"))
        .arg(arg!(-p --port <PORT> "Port for the server").value_parser(value_parser!(u16)).default_value("3000"))
        .arg(arg!(-l --log ... "Show api logs. Use multiple times for highier debuging level"))
        .arg(arg!(-d --dir <DIR> "Main folder for storing files").value_parser(value_parser!(PathBuf)).default_value("files"))
        .arg(arg!(-n --"disable-login" "No one will be allowed to create new account"))
        .get_matches();

    let config = Config {
        stop_web: *matches.get_one::<bool>("stop-web").unwrap_or(&false),
        address: *matches.get_one::<Ipv4Addr>("address").unwrap_or(&Ipv4Addr::new(127, 0, 0, 1)),
        port: *matches.get_one::<u16>("port").unwrap_or(&3000),
        log: *matches.get_one::<u8>("log").unwrap_or(&2),
        dir: matches.get_one::<PathBuf>("dir").unwrap_or(&PathBuf::from("files")).clone(),
        disable_login: *matches.get_one::<bool>("disable-login").unwrap(),
    };

    fs::create_dir_all(config.dir.clone())?;

    env_logger::init_from_env(env_logger::Env::new().default_filter_or(config.log.get_from_u8().as_str()));

    eprintln!("{}", r"
   __    _   __   __    __            _____   ___    _  __
  / /   (_) / /_ / /_  / / ___       / ___/  / _ \  / |/ /
 / /__ / / / __// __/ / / / -_)     / /__   / // / /    /
/____//_/  \__/ \__/ /_/  \__/      \___/  /____/ /_/|_/
    ".red().bold());

    Ok(config)
}
