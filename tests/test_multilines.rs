use clap::Clap;
use code_it_later_rs::{
    config::Args,
    fs_operation::{Bread, Crumb},
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
        fs_operation::handle_files(&conf).collect::<Vec<_>>(),
        vec![Bread::new(
            "./tests/testcases/multilines.rs".to_string(),
            vec![
                Crumb::new(1, None, "line1 line2 and line3".to_string()),
                Crumb::new(5, None, "line4 is diffrent and line5".to_string()),
                Crumb::new(7, None, "line6".to_string())
            ]
        )]
    )
}
