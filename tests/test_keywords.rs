use clap::Parser;
use code_it_later_rs::{
    args::*,
    datatypes::{Bread, Crumb},
    *,
};

#[test]
fn test_keywords() {
    let args = Args::parse_from(vec![
        "codeitlater",
        "-x",
        "target",
        "-k",
        "TODO",
        "./tests/testcases/keywords.lisp",
    ]);

    let conf = config::Config::from(&args);
    //dbg!(&conf);
    assert_eq!(
        fs_operation::handle_files(conf).collect::<Vec<_>>(),
        vec![Bread::new(
            "./tests/testcases/keywords.lisp".to_string(),
            vec![Crumb::new(
                1,
                0,
                Some("TODO".to_string()),
                "this is TODO".to_string()
            ),]
        )]
    );
}

#[test]
fn test_ignore_keyword_file() {
    let args = Args::parse_from(vec![
        "codeitlater",
        "-x",
        "target",
        "-k",
        "TODO",
        "./tests/testcases/test.rs",
    ]);

    let conf = config::Config::from(&args);

    assert_eq!(
        fs_operation::handle_files(conf).collect::<Vec<_>>(),
        vec![Bread::new(
            "./tests/testcases/test.rs".to_string(),
            vec![Crumb::new(
                6,
                0,
                Some("TODO".to_string()),
                "this is the ignore line".to_string()
            )
            .add_ignore_flag()]
        )]
    );

    let args = Args::parse_from(vec![
        "codeitlater",
        "-x",
        "target",
        "-k",
        "MARK",
        "./tests/testcases/keywords.lisp",
    ]);

    let conf = config::Config::from(&args);

    assert_eq!(
        fs_operation::handle_files(conf).collect::<Vec<_>>(),
        vec![Bread::new(
            "./tests/testcases/keywords.lisp".to_string(),
            vec![
                Crumb::new(3, 0, Some("MARK".to_string()), "this is MARK".to_string()),
                Crumb::new(
                    4,
                    0,
                    Some("MARK".to_string()),
                    "this is ignored MARK".to_string()
                )
                .add_ignore_flag()
            ]
        )]
    );
}
