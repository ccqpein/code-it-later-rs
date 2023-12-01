use clap::Parser;
use code_it_later_rs::{args::*, *};
use lazy_static::lazy_static;
use std::fs::{self, copy, remove_file};
use std::io::{prelude::*, BufReader, Result};
use std::path::Path;
use std::sync::Mutex;

fn same_file(file0: impl AsRef<Path>, file1: impl AsRef<Path>) -> Result<bool> {
    let f0 = fs::File::open(&file0)?;
    let f1 = fs::File::open(&file1)?;
    let reader0 = BufReader::new(f0).lines();
    let reader1 = BufReader::new(f1).lines();

    Ok(reader0.zip(reader1).all(|(a, b)| a.unwrap() == b.unwrap()))
}

lazy_static! {
    static ref TEST_CLEAN_LOCK: Mutex<()> = Mutex::new(());
}

#[test]
fn test_delete_the_crumbs() -> Result<()> {
    let _lock = TEST_CLEAN_LOCK.lock();
    copy(
        "./tests/testcases/clean_case_0.rs.bkp",
        "./tests/testcases/clean_case_0.rs",
    )?;

    let args = Args::parse_from(vec!["codeitlater", "./tests/testcases/clean_case_0.rs"]);
    let conf = config::Config::from(&args);

    let mut bread = fs_operation::handle_files(conf);
    fs_operation::delete_the_crumbs(bread.next().unwrap())?;
    assert!(same_file(
        "tests/testcases/clean_case_0.rs.delete_expect",
        "tests/testcases/clean_case_0.rs",
    )
    .unwrap());

    remove_file("./tests/testcases/clean_case_0.rs")
}

#[test]
fn test_restore_the_crumbs() -> Result<()> {
    let _lock = TEST_CLEAN_LOCK.lock();
    copy(
        "./tests/testcases/clean_case_0.rs.bkp",
        "./tests/testcases/clean_case_0.rs",
    )?;
    let args = Args::parse_from(vec![
        "codeitlater",
        //"-k", // this can just restore the ignore TODO
        //"TODO",
        "./tests/testcases/clean_case_0.rs",
    ]);
    let conf = config::Config::from(&args);

    let mut bread = fs_operation::handle_files(conf);
    fs_operation::restore_the_crumb(bread.next().unwrap())?;

    assert!(same_file(
        "tests/testcases/clean_case_0.rs.restore_expect",
        "tests/testcases/clean_case_0.rs",
    )
    .unwrap());

    remove_file("./tests/testcases/clean_case_0.rs")
}

//#[test]
//:= fmt command change, this test case need to fix in future
// fn test_fmt_after_clean() -> Result<()> {
//     copy(
//         "./tests/testcases/format_test.go.bkp",
//         "./tests/testcases/format_test.go",
//     )?;

//     let args = Args::parse_from(vec!["codeitlater", "./tests/testcases/format_test.go"]);
//     let conf = config::Config::from(&args);

//     let mut bread = fs_operation::handle_files(conf);
//     let file_path = fs_operation::clean_the_crumbs(bread.next().unwrap())?;

//     assert!(run_format_command_to_file("go fmt", vec![file_path]).is_ok());

//     assert!(same_file(
//         "tests/testcases/format_test.go.expect",
//         "tests/testcases/format_test.go",
//     )
//     .unwrap());

//     remove_file("./tests/testcases/format_test.go")
// }
