use clap::Parser;
use code_it_later_rs::{
    args::*,
    datatypes::{Bread, Crumb},
    *,
};

#[test]
fn test_rs_file() {
    let args = Args::parse_from(vec!["codeitlater", "-x", "target", "./tests/testcases/test.rs"]);

    let conf = config::Config::from(&args);

    assert_eq!(
        fs_operation::handle_files(conf).collect::<Vec<_>>(),
        vec![Bread::new(
            "./tests/testcases/test.rs".to_string(),
            vec![
                Crumb::new(1, 0, "this is rust".to_string(), "/*".to_string()),
                Crumb::new(4, 0, "this is also rust".to_string(), "///".to_string())
            ]
        )]
    );
}

#[test]
fn test_py_file() {
    let args = Args::parse_from(vec![
        "codeitlater",
        "-x",
        "target",
        "--",
        "./tests/testcases/test.py",
    ]);

    let conf = config::Config::from(&args);

    assert_eq!(
        fs_operation::handle_files(conf).collect::<Vec<_>>(),
        vec![Bread::new(
            "./tests/testcases/test.py".to_string(),
            vec![Crumb::new(
                1,
                0,
                "this is python".to_string(),
                "#".to_string()
            ),]
        )]
    );
}

#[test]
fn test_go_file() {
    let args = Args::parse_from(vec![
        "codeitlater",
        "-x",
        "target",
        "--",
        "./tests/testcases/test.go",
    ]);

    let conf = config::Config::from(&args);

    assert_eq!(
        fs_operation::handle_files(conf).collect::<Vec<_>>(),
        vec![Bread::new(
            "./tests/testcases/test.go".to_string(),
            vec![
                Crumb::new(
                    3,
                    0,
                    "this line can be read by codeitlater".to_string(),
                    "//".to_string()
                ),
                Crumb::new(
                    4,
                    0,
                    "MARK: you can left keyword to marked comment line".to_string(),
                    "//".to_string()
                )
            ]
        )]
    );
}
