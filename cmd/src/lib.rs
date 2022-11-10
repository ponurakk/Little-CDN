use clap::Parser;
use colored::Colorize;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Don't run the web view
    #[arg(short, long)]
    stop_web: bool,

    /// Address on which server should run
    #[arg(short, long, default_value = "127.0.0.1")]
    address: String,

    /// Port for the server
    #[arg(short, long, default_value = "3000")]
    port: String,

    /// Show api logs
    #[arg(short, long)]
    log: bool,

    /// Main folder
    #[arg(short, long, default_value = "files")]
    dir: String,
}

pub fn main() {
    println!("{}", r"
   __    _   __   __    __            _____   ___    _  __
  / /   (_) / /_ / /_  / / ___       / ___/  / _ \  / |/ /
 / /__ / / / __// __/ / / / -_)     / /__   / // / /    /
/____//_/  \__/ \__/ /_/  \__/      \___/  /____/ /_/|_/
    ".red().bold());
    let args: Args = Args::parse();

    // println!("{:#?}", &args);

    if !args.stop_web {
        let server = cdn_server_api::main(args.address, args.port.parse().unwrap(), args.log, args.dir);
        match server {
            Ok(v) => v,
            Err(e) => panic!("{}", e),
        }
    }
}