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
                    let mut cc = Crumb::new(1, 0, "line1...".to_string());
                    cc.add_tail(Crumb::new(2, 0, String::from("line2...")));
                    cc.add_tail(Crumb::new(3, 0, String::from("and line3")));
                    cc
                },
                {
                    let mut cc = Crumb::new(5, 0, "line4 is diffrent...".to_string());
                    cc.add_tail(Crumb::new(6, 0, String::from("and line5")));
                    cc
                },
                Crumb::new(7, 0, "line6".to_string())
            ]
        )]
    )
}
