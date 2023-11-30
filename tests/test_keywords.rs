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
    let c =
        Crumb::new_for_test(
            1,
            0,
            vec![],
            Some("TODO".to_string()),
            "this is TODO".to_string(),
            "TODO: this is TODO".to_string(),
            ";;;;;;;;".to_string(),
            false,
        );

    assert_eq!(
        fs_operation::handle_files(conf).collect::<Vec<_>>(),
        vec![Bread::new(
            "./tests/testcases/keywords.lisp".to_string(),
            vec![c,]
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

    let c =
        Crumb::new_for_test(
            6,
            0,
            vec![],
            Some("TODO".to_string()),
            "this is the ignore line".to_string(),
            "!TODO: this is the ignore line".to_string(),
            "//".to_string(),
            true,
        );

    assert_eq!(
        fs_operation::handle_files(conf).collect::<Vec<_>>(),
        vec![Bread::new("./tests/testcases/test.rs".to_string(), vec![c])]
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
                Crumb::new_for_test(
                    3,
                    0,
                    vec![],
                    Some("MARK".to_string()),
                    "this is MARK".to_string(),
                    "MARK: this is MARK".to_string(),
                    ";;".to_string(),
                    false
                ),
                Crumb::new_for_test(
                    4,
                    0,
                    vec![],
                    Some("MARK".to_string()),
                    "this is ignored MARK".to_string(),
                    "!MARK: this is ignored MARK".to_string(),
                    ";;".to_string(),
                    true
                ),
            ]
        )]
    );
}
