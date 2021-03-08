mod app;
mod cli;
mod indent;
mod output;
mod pathtree;
mod report;
mod utils;

use structopt::StructOpt;

fn main() {
    app::main(&cli::Args::from_args());
}
