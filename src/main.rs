use clap::Parser;
use code_it_later_rs::{
    args::Args,
    config::{self, parse_from_current_path_config},
};

fn main() {
    let commandline_args = Args::parse();

    #[cfg(debug_assertions)]
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

    #[cfg(debug_assertions)]
    dbg!(&args, &conf);

    code_it_later_rs::prompt(conf).unwrap();
}
