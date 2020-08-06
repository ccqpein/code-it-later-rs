use clap::Clap;
use code_it_later_rs::{config, fs_operation};

fn main() {
    let args = config::Args::parse();
    let conf = config::Config::from(&args);
    fs_operation::handle_files(&conf);
}
