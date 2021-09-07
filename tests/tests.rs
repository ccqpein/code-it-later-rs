use clap::Clap;
use code_it_later_rs::{config::Args, *};

#[test]
fn test_rs_file() {
    let args = Args::parse_from(vec![
        "codeitlater",
        "-x",
        "target",
        "-d",
        "./tests/testcases",
    ]);

    let conf = config::Config::from(&args);
    println!(
        "{:?}",
        fs_operation::handle_files(&conf).collect::<Vec<_>>()
    );
}
