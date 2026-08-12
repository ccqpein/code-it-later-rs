use clap::Parser;
use code_it_later_rs::{
    args::*,
    datatypes::{Bread, Crumb},
    *,
};

#[test]
fn test_multilines() {
    let args = Args::parse_from(vec![
        "codeitlater",
        "-x",
        "target",
        "--",
        "./tests/testcases/multilines.rs",
    ]);

    let conf = config::Config::from(&args);

    assert_eq!(
        fs_operation::handle_files(conf).collect::<Vec<_>>(),
        vec![Bread::new(
            "./tests/testcases/multilines.rs".to_string(),
            vec![
                {
                    let mut cc = Crumb::new(1, 0, "line1...".to_string(), "//".to_string(), String::new());
                    cc.add_tail(Crumb::new(2, 0, String::from("line2..."), "//".to_string(), String::new()));
                    cc.add_tail(Crumb::new(
                        3,
                        0,
                        String::from("and line3"),
                        "//".to_string(),
                        String::new(),
                    ));
                    cc
                },
                {
                    let mut cc =
                        Crumb::new(5, 0, "line4 is diffrent...".to_string(), "//".to_string(), String::new());
                    cc.add_tail(Crumb::new(
                        6,
                        0,
                        String::from("and line5"),
                        "//".to_string(),
                        String::new(),
                    ));
                    cc
                },
                Crumb::new(7, 0, "line6".to_string(), "//".to_string(), String::new())
            ]
        )]
    )
}

#[test]
fn test_go_multilines() {
    let args = Args::parse_from(vec![
        "codeitlater",
        "-x",
        "target",
        "--",
        "./tests/testcases/go_multiline.go",
    ]);

    let conf = config::Config::from(&args);

    assert_eq!(
        fs_operation::handle_files(conf).collect::<Vec<_>>(),
        vec![Bread::new(
            "./tests/testcases/go_multiline.go".to_string(),
            vec![
                {
                    let mut cc = Crumb::new(1, 0, "".to_string(), "/*".to_string(), String::new());
                    cc.add_tail(
                        Crumb::new(2, 2, "comment".to_string(), String::new(), String::new()).with_has_tail(true),
                    );
                    cc.add_tail(
                        Crumb::new(3, 0, "aaaa".to_string(), String::new(), String::new()).with_has_tail(true),
                    );
                    cc.add_tail(
                        Crumb::new(4, 0, "fff".to_string(), String::new(), String::new()).with_has_tail(true),
                    );
                    cc.add_tail(Crumb::new(5, 0, "yo".to_string(), String::new(), "*/".to_string()));
                    cc.with_has_tail(false)
                },
                // key words test
                {
                    let mut cc = Crumb::new(7, 0, "TODO:".to_string(), "/*".to_string(), String::new());
                    cc.add_tail(
                        Crumb::new(8, 2, "hello".to_string(), String::new(), String::new()).with_has_tail(true),
                    );
                    cc.add_tail(Crumb::new(9, 0, "".to_string(), String::new(), "*/".to_string()));
                    cc.with_has_tail(false)
                }
            ]
        )]
    );
}
