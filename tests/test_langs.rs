use clap::Parser;
use code_it_later_rs::{
    args::*,
    datatypes::{Bread, Crumb},
    *,
};

#[test]
fn test_rs_file() {
    let args = Args::parse_from(vec![
        "codeitlater",
        "-x",
        "target",
        "./tests/testcases/test.rs",
    ]);

    let conf = config::Config::from(&args);
    let mut x = Crumb::new(
        1,
        0,
        "this is rust".to_string(),
        "/*".to_string(),
        String::new(),
    )
    .with_has_tail(true);

    x.add_tail(Crumb::new(
        2,
        1,
        "".to_string(),
        String::new(),
        "*/".to_string(),
    ));

    assert_eq!(
        fs_operation::handle_files(conf).collect::<Vec<_>>(),
        vec![Bread::new(
            "./tests/testcases/test.rs".to_string(),
            vec![
                x,
                Crumb::new(
                    4,
                    0,
                    "this is also rust".to_string(),
                    "///".to_string(),
                    String::new()
                )
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
                "#".to_string(),
                String::new()
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
                    "//".to_string(),
                    String::new()
                ),
                Crumb::new(
                    4,
                    0,
                    "MARK: you can left keyword to marked comment line".to_string(),
                    "//".to_string(),
                    String::new()
                )
            ]
        )]
    );
}

#[test]
fn test_hs_file() {
    let args = Args::parse_from(vec![
        "codeitlater",
        "-x",
        "target",
        "--",
        "./tests/testcases/test.hs",
    ]);

    let conf = config::Config::from(&args);

    assert_eq!(
        fs_operation::handle_files(conf).collect::<Vec<_>>(),
        vec![Bread::new(
            "./tests/testcases/test.hs".to_string(),
            vec![
                Crumb::new(
                    1,
                    0,
                    "comment1".to_string(),
                    "--".to_string(),
                    String::new()
                ),
                Crumb::new(
                    2,
                    0,
                    "comment2".to_string(),
                    "-- ".to_string(),
                    String::new()
                )
            ]
        )]
    );
}

#[test]
fn test_makefile_extensionless() {
    let args = Args::parse_from(vec![
        "codeitlater",
        "-x",
        "target",
        "--",
        "./tests/testcases/Makefile",
    ]);

    let conf = config::Config::from(&args);

    assert_eq!(
        fs_operation::handle_files(conf).collect::<Vec<_>>(),
        vec![Bread::new(
            "./tests/testcases/Makefile".to_string(),
            vec![Crumb::new(
                1,
                0,
                "comment in makefile".to_string(),
                "#".to_string(),
                String::new()
            )]
        )]
    );
}

#[test]
fn test_explicit_extensionless_fallback() {
    let args = Args::parse_from(vec![
        "codeitlater",
        "-x",
        "target",
        "--",
        "./tests/testcases/no_ext_explicit",
    ]);

    let conf = config::Config::from(&args);

    assert_eq!(
        fs_operation::handle_files(conf).collect::<Vec<_>>(),
        vec![Bread::new(
            "./tests/testcases/no_ext_explicit".to_string(),
            vec![Crumb::new(
                1,
                0,
                "explicit fallback comment".to_string(),
                "//".to_string(),
                String::new()
            )]
        )]
    );
}
