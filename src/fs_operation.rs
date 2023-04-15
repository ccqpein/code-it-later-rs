use super::config::{Config, KEYWORDS_REGEX, REGEX_TABLE};
use super::datatypes::*;
use log::debug;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::ffi::OsString;
use std::fs::{self, read_dir, OpenOptions};
use std::io::{self, prelude::*, BufReader};
use std::process::Command;
use std::sync::{Arc, RwLock};
use std::{io::Result, path::Path, path::PathBuf, thread};

/// Vector of all pathbufs
type Dirs = Vec<PathBuf>;

/// File struct, including file path and the &Regex of this file
/// &Regex CANNOT be nil
#[derive(Debug)]
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
    match re.find(line) {
        Some(mat) => {
            let position = mat.start();
            let content = re.captures(line).unwrap()[2].to_string();
            Some(Crumb::new(line_num, position, None, content))
        }
        None => None,
    }
}

/// Operate this file
fn op_file(file: File, kwreg: &Option<Regex>, conf: Arc<RwLock<Config>>) -> Result<Option<Bread>> {
    let breads = match bake_bread(&file, kwreg) {
        Ok(b) => b,
        Err(e) => {
            debug!("file {} had error {}", file.to_string(), e.to_string());
            return Ok(None);
        }
    };

    if !conf.read().unwrap().delete {
        Ok(breads)
    } else {
        match breads {
            Some(bb) => {
                clean_the_crumbs(bb)?;
                Ok(None)
            }
            None => Ok(None),
        }
    }
}

/// make bread for this file
fn bake_bread(file: &File, kwreg: &Option<Regex>) -> Result<Option<Bread>> {
    // start to read file
    let mut buf = vec![];
    let file_p = file.to_string();
    let mut f: std::fs::File = std::fs::File::open(file.0.clone())?;
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
                                h.add_tail(cb);
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

/// clean crumbs and re-write the file
pub fn clean_the_crumbs(Bread { file_path, crumbs }: Bread) -> Result<String> {
    let all_delete_line_postion_pairs = crumbs
        .iter()
        .map(|crumb| crumb.all_lines_num_postion_pair())
        .flatten();

    delete_lines_on(&file_path, all_delete_line_postion_pairs)?;

    println!("cleaned the crumbs in {}", file_path);
    Ok(file_path)
}

/// clean crumbs by special indexes
pub fn clean_the_crumbs_on_special_index(
    Bread { file_path, crumbs }: Bread,
    indexes: HashSet<usize>,
) -> Result<String> {
    let mut all_delete_lines = vec![];
    for ind in &indexes {
        match crumbs.get(*ind) {
            Some(c) => all_delete_lines.append(&mut c.all_lines_num_postion_pair()),
            None => return Err(io::Error::other("cannot find crumb index in bread")),
        }
    }

    delete_lines_on(&file_path, all_delete_lines.into_iter())?;

    println!("cleaned {} crumbs in {}", indexes.len(), file_path);

    Ok(file_path)
}

/// delete special lines of the file on file_path
fn delete_lines_on(
    file_path: &str,
    line_num_pos_pairs: impl Iterator<Item = (usize, usize)>,
) -> Result<()> {
    let f = fs::File::open(&file_path)?;
    let reader = BufReader::new(f).lines();

    let all_delete_lines = line_num_pos_pairs.collect();

    let finish_deleted = delete_nth_lines(reader, all_delete_lines)?
        .into_iter()
        .map(|line| line.into_bytes());

    let mut new_file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(file_path.clone())?;

    for line in finish_deleted {
        new_file.write_all(&line)?;
        new_file.write_all(b"\n")?
    }
    Ok(())
}

/// delete crumbs of file, return the new file contents without the crumbs deleted
fn delete_nth_lines(
    f: impl Iterator<Item = Result<String>>,
    nm: HashMap<usize, usize>,
) -> Result<Vec<String>> {
    let mut result = vec![];

    for (line_num, ll) in f.enumerate() {
        if nm.contains_key(&(line_num + 1)) {
            let mut new_l = ll?;
            new_l.truncate(*nm.get(&(line_num + 1)).unwrap());
            if new_l == "" {
                // empty line just skip
                continue;
            }
            result.push(new_l);
        } else {
            result.push(ll?);
        }
    }

    Ok(result)
}

/// run format command with filepath input
pub fn run_format_command_to_file(
    fmt_command: &str,
    files: impl IntoIterator<Item = String>,
) -> std::result::Result<(), String> {
    let mut command_splits = fmt_command.split(' ');
    let first = command_splits
        .next()
        .ok_or("fmt_command cannot be emptye".to_string())?;

    let mut comm = Command::new(first);
    let mut child = comm
        .args(command_splits)
        //:= TODO: change doc
        //.args(files) // add files at the endding
        .spawn()
        .expect("Cannot run the fmt_command");

    child
        .wait()
        .expect("fmt command wasn't running")
        .exit_ok()
        .map_err(|e| e.to_string())
}

/// entry function of main logic
pub fn handle_files(conf: Config) -> impl Iterator<Item = Bread> {
    // first add all files in arguments
    let mut all_files: Vec<File> = files_in_dir_or_file_vec(&conf.files, &conf).unwrap();

    // split to groups
    let threads_num = 24;
    let len = all_files.len();
    let count = len / threads_num;
    let mut groups: Vec<Vec<File>> = vec![];
    for _ in 0..threads_num - 1 {
        groups.push(all_files.drain(0..count).collect())
    }
    groups.push(all_files.drain(0..).collect());

    let conf = Arc::new(RwLock::new(conf));
    groups
        .into_iter()
        .map(move |fs| {
            let kwreg = KEYWORDS_REGEX.lock().unwrap().clone();
            let conf_c = Arc::clone(&conf);
            thread::spawn(|| {
                fs.into_iter()
                    .filter_map(move |f| op_file(f, &kwreg, conf_c.clone()).unwrap())
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
