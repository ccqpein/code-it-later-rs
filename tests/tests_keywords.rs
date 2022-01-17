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
        "--",
        "./tests/testcases/keywords.lisp",
    ]);

    let conf = config::Config::from(&args);
    //dbg!(&conf);
    assert_eq!(
        fs_operation::handle_files(&conf).collect::<Vec<_>>(),
        vec![Bread::new(
            "./tests/testcases/keywords.lisp".to_string(),
            vec![Crumb::new(
                1,
                Some("TODO".to_string()),
                "this is TODO".to_string()
            ),]
        )]
    );
}
