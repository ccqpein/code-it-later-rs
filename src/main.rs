use clap::Clap;
use code_it_later_rs::{
    config::{self, parse_from_current_path_config},
    fs_operation,
};

fn main() {
    let commandline_args = config::Args::parse();
    dbg!(&commandline_args);
    let args = match parse_from_current_path_config() {
        // if have local config
        Some(mut local_conf) => {
            local_conf.union(commandline_args); // local union with commond line input
            local_conf
        }
        None => commandline_args,
    };

    let conf = config::Config::from(&args);
    dbg!(&args, &conf);
    fs_operation::handle_files(&conf).for_each(|b| println!("{}", b));
}
