use structopt::StructOpt;

mod app;
mod cli;
mod indent;
mod output;
mod pathtree;
mod report;
mod utils;

fn main() {
    app::main(&cli::Args::from_args());
}
