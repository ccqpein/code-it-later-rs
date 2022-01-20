use clap::Parser;
use code_it_later_rs::{args::*, *};
use std::fs::{self, copy, remove_file};
use std::io::{prelude::*, BufReader, Result};
use std::path::Path;

fn same_file(file0: impl AsRef<Path>, file1: impl AsRef<Path>) -> Result<bool> {
    let f0 = fs::File::open(&file0)?;
    let f1 = fs::File::open(&file1)?;
    let reader0 = BufReader::new(f0).lines();
    let reader1 = BufReader::new(f1).lines();

    Ok(reader0.zip(reader1).all(|(a, b)| a.unwrap() == b.unwrap()))
}

#[test]
fn test_clean_the_crumbs() -> Result<()> {
    copy(
        "./tests/testcases/clean_case_0.rs.bkp",
        "./tests/testcases/clean_case_0.rs",
    )?;

    let args = Args::parse_from(vec!["codeitlater", "./tests/testcases/clean_case_0.rs"]);
    let conf = config::Config::from(&args);

    let mut bread = fs_operation::handle_files(conf);
    fs_operation::clean_the_crumbs(bread.next().unwrap());
    assert!(same_file(
        "tests/testcases/clean_case_0.rs.expect",
        "./tests/testcases/clean_case_0.rs",
    )
    .unwrap());

    remove_file("./tests/testcases/clean_case_0.rs")
}
