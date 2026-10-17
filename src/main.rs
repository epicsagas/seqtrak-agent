use clap::Parser;
use seqtrak_agent::cli::{self, Cli};

fn main() {
    let cli = Cli::parse();
    match cli::run(&cli) {
        Ok(msg) => println!("{msg}"),
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    }
}
