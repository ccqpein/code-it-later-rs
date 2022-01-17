use super::config::{Config, KEYWORDS_REGEX, REGEX_TABLE};
use super::datatypes::*;
use regex::Regex;
use std::ffi::OsString;
use std::fs::read_dir;
use std::io::prelude::*;
use std::{io::Result, path::Path, path::PathBuf, thread};

/// Vector of all pathbufs
type Dirs = Vec<PathBuf>;

/// File struct, including file path and the &Regex of this file
/// &Regex CANNOT be nil
//#[derive(Debug)]
struct File(PathBuf, &'static Regex);

impl File {
    /// Return string of file path
    fn to_string(&self) -> String {
        self.0.as_os_str().to_os_string().into_string().unwrap()
    }
}

type Files = Vec<File>;

/// loop all string inside paths_or_files, if it is file, store it, if it is dir
/// store all files inside thsi dir (recursivly)
fn files_in_dir_or_file_vec(paths_or_files: &[impl AsRef<Path>], conf: &Config) -> Result<Files> {
    let mut result: Files = vec![];
    for ele in paths_or_files {
        if ele.as_ref().is_dir() {
            result.append(&mut all_files_in_dir(ele, conf)?)
        } else {
            file_checker(
                &mut result,
                ele.as_ref(),
                &conf.filetypes,
                conf.filetypes.len(),
            )
        }
    }
    Ok(result)
}

/// Find all files in this dir recursivly
fn all_files_in_dir<T>(p: T, conf: &Config) -> Result<Files>
where
    T: AsRef<Path>,
{
    let mut result = vec![];
    let (mut files, dirs) = files_and_dirs_in_path(p, &conf)?;
    result.append(&mut files);

    if dirs.len() != 0 {
        result.append(
            &mut dirs
                .iter()
                .map(|d| all_files_in_dir(d, conf).unwrap())
                .flatten()
                .collect::<Files>(),
        )
    }

    Ok(result)
}

/// Find files and dirs in this folder
fn files_and_dirs_in_path(p: impl AsRef<Path>, conf: &Config) -> Result<(Files, Dirs)> {
    let (mut f, mut d): (Files, Dirs) = (vec![], vec![]);

    // get filetypes
    let filetypes = &conf.filetypes;
    let filetypes_count = filetypes.len();

    // get ignore dirs
    let ignore_dirs = &conf.ignore_dirs;
    let ignore_dirs_count = ignore_dirs.len();

    for entry in read_dir(p)? {
        let dir = entry?;
        let path = dir.path();

        if path.is_dir() {
            // check ignore dirs
            if ignore_dirs_count != 0 {
                if let Some(d_name) = path.file_name() {
                    if !ignore_dirs.contains(&d_name.to_os_string()) {
                        d.push(path)
                    }
                }
            } else {
                d.push(path)
            }
        } else {
            file_checker(&mut f, &path, &filetypes, filetypes_count)
        }
    }
    Ok((f, d))
}

/// if file path pass check, add it to files
fn file_checker(files: &mut Files, path: &Path, filetypes: &[OsString], filetypes_count: usize) {
    // check filetypes
    if filetypes_count != 0 {
        // special filetypes
        if let Some(t) = path.extension() {
            // file has extension
            if filetypes.contains(&t.to_os_string()) {
                // this file include in filetypes
                let aa = REGEX_TABLE.lock();
                if let Some(re) = aa.as_ref().unwrap().get(t.to_str().unwrap()) {
                    // and has regex for this type
                    let re = unsafe {
                        match (re as *const Regex).clone().as_ref() {
                            Some(a) => a,
                            None => return,
                        }
                    };
                    files.push(File(path.to_path_buf(), re))
                }
            }
        }
    } else {
        if let Some(t) = path.extension() {
            // file has extension
            let aa = REGEX_TABLE.lock();
            if let Some(re) = aa.as_ref().unwrap().get(t.to_str().unwrap()) {
                // and has regex for this type
                let re = unsafe {
                    match (re as *const Regex).clone().as_ref() {
                        Some(a) => a,
                        None => return,
                    }
                };
                files.push(File(path.to_path_buf(), re))
            }
        }
    }
}

/// Filter this line
fn filter_line(line: &str, line_num: usize, re: &Regex) -> Option<Crumb> {
    match re.captures(line) {
        Some(content) => Some(Crumb {
            line_num,
            keyword: None,
            content: content[2].to_string(),
        }),
        None => None,
    }
}

/// Operate this file
fn op_file(file: File, kwreg: &Option<Regex>) -> Result<Option<Bread>> {
    // start to read file
    let mut buf = vec![];
    let file_p = file.to_string();
    let mut f: std::fs::File = std::fs::File::open(file.0)?;
    f.read_to_end(&mut buf)?;

    let mut line_num = 0;
    let mut ss = String::new(); // temp
    let mut buf = buf.as_slice();
    let mut result = vec![];
    let mut head: Option<Crumb> = None; // for tail support

    // closure for keywords feature
    let mut keyword_checker_and_push = |mut cb: Crumb| {
        if kwreg.is_some() {
            if cb.filter_keywords(kwreg.as_ref().unwrap()) {
                result.push(cb)
            }
        } else {
            result.push(cb)
        }
    };

    loop {
        line_num += 1;
        match buf.read_line(&mut ss) {
            Ok(0) | Err(_) => {
                if head.is_some() {
                    keyword_checker_and_push(head.unwrap());
                }
                break; // if EOF or any error in this file, break
            }
            Ok(_) => match filter_line(&ss, line_num, file.1) {
                Some(cb) => {
                    // check head first
                    match head {
                        Some(ref mut h) => {
                            if h.has_tail() {
                                // if head has tail, add this line to head, continue
                                h.add_tail(&cb);
                                ss.clear(); // before continue, clear temp
                                continue;
                            } else {
                                // store head
                                keyword_checker_and_push(head.unwrap());
                                head = None;
                            }
                        }
                        None => (),
                    }

                    if cb.has_tail() {
                        // make new head
                        head = Some(cb);
                    } else {
                        // store result
                        keyword_checker_and_push(cb)
                    }
                }
                None => {
                    if head.is_some() {
                        keyword_checker_and_push(head.unwrap());
                        head = None;
                    }
                }
            },
        }
        ss.clear()
    }

    if result.len() == 0 {
        Ok(None)
    } else {
        Ok(Some(Bread::new(file_p, result)))
    }
}

/// entry function of main logic
pub fn handle_files(conf: &Config) -> impl Iterator<Item = Bread> {
    // first add all files in arguments
    let mut all_files: Vec<File> = files_in_dir_or_file_vec(&conf.files, conf).unwrap();

    // split to groups
    let threads_num = 24;
    let len = all_files.len();
    let count = len / threads_num;
    let mut groups: Vec<Vec<File>> = vec![];
    for _ in 0..threads_num - 1 {
        groups.push(all_files.drain(0..count).collect())
    }
    groups.push(all_files.drain(0..).collect());

    groups
        .into_iter()
        .map(|fs| {
            let kwreg = KEYWORDS_REGEX.lock().unwrap().clone();
            thread::spawn(move || {
                fs.into_iter()
                    .map(|f| op_file(f, &kwreg).unwrap())
                    .filter(|r| r.is_some())
                    .map(|q| q.unwrap())
                    .collect::<Vec<Bread>>()
            })
        })
        .map(|han| han.join().unwrap())
        .flatten()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_files_and_dirs_in_path() -> Result<()> {
        let (fs, dirs) = files_and_dirs_in_path("./tests/testcases", &Default::default())?;

        assert_eq!(dirs.len(), 0);
        assert_eq!(fs[0].0, PathBuf::from("./tests/testcases/multilines.rs"),);
        Ok(())
    }
}
