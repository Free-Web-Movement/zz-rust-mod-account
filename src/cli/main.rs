mod cli;
mod repl;

use clap::Parser;
use cli::{Cli, run_cli};


fn main() {
    let cli = Cli::parse();
    run_cli(cli);
}
