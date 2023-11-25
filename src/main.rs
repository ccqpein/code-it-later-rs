#![feature(let_chains)]

use clap::Parser;
use code_it_later_rs::{
    args::{parse_from_current_path_config, Args},
    config,
    fs_operation::run_format_command_to_file,
};

fn main() -> Result<(), String> {
    let commandline_args = Args::parse();
    env_logger::init();

    #[cfg(debug_assertions)]
    dbg!(&commandline_args);

    let args =
        match parse_from_current_path_config(commandline_args.config_location()) {
            // if have local config
            Some(mut local_conf) => {
                local_conf.cover(commandline_args); // local union with commond line input
                local_conf
            }
            None => commandline_args,
        };

    let conf = config::Config::from(&args);

    #[cfg(debug_assertions)]
    dbg!(&args, &conf);

    if let Some(files_changed) = code_it_later_rs::prompt(conf)?
        && let Some(fmt) = args.fmt_command()
    {
        run_format_command_to_file(fmt, files_changed)?
    };

    Ok(())
}
