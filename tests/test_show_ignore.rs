use clap::Parser;
use code_it_later_rs::{
    args::Args,
    config,
    datatypes::{Bread, Crumb},
    fs_operation,
};

#[test]
fn test_show_ignore() {
    let args = Args::parse_from(vec![
        "codeitlater",
        "-x",
        "target",
        "--show-ignored",
        "true",
        "./tests/testcases/test.rs",
    ]);

    let conf = config::Config::from(&args);
    assert_eq!(
        fs_operation::handle_files(conf).collect::<Vec<_>>(),
        vec![Bread::new(
            "./tests/testcases/test.rs".to_string(),
            vec![
                Crumb::new(1, 0, "this is rust".to_string()),
                Crumb::new(4, 0, "this is also rust".to_string()),
                Crumb::new(6, 0, "!TODO: this is the ignore line".to_string()).add_ignore_flag()
            ]
        )]
    );
}
