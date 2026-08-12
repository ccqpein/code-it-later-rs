use crate::config::REGEX_TABLE_MUL;

use super::config::{Config, FALLBACK_REGEX, KEYWORDS_REGEX, REGEX_TABLE};
use super::datatypes::*;
use log::debug;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::ffi::OsString;
use std::fs::{self, OpenOptions, read_dir};
use std::io::{self, BufReader, prelude::*};
use std::num::NonZeroUsize;
use std::process::Command;
use std::sync::{Arc, RwLock};
use std::{io::Result, path::Path, path::PathBuf, thread};

/// how many thread when it runs
const THREAD_NUM: Option<NonZeroUsize> = NonZeroUsize::new(4);

/// Vector of all pathbufs
type Dirs = Vec<PathBuf>;

/// File struct, including file path and the &Regex of this file
/// &Regex CANNOT be nil
#[derive(Debug)]
struct File(
    PathBuf,
    Option<&'static Regex>,
    Option<&'static (Regex, Regex)>,
);

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
                true,
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
            file_checker(&mut f, &path, &filetypes, filetypes_count, false)
        }
    }
    Ok((f, d))
}

/// if file path pass check, add it to files
fn file_checker(
    files: &mut Files,
    path: &Path,
    filetypes: &[OsString],
    filetypes_count: usize,
    is_explicit: bool,
) {
    let ext = path.extension();
    let file_name = path.file_name();
    let ext_str = ext.and_then(|t| t.to_str());
    let file_name_str = file_name.and_then(|f| f.to_str()).map(|s| s.to_lowercase());

    // check filetypes
    if filetypes_count != 0 {
        // special filetypes
        if let Some(t) = ext {
            // file has extension
            if filetypes.contains(&t.to_os_string()) {
                // this file include in filetypes
                let aa = REGEX_TABLE.lock();
                let bb = REGEX_TABLE_MUL.lock();

                if let Some(t_str) = ext_str {
                    let single_line_re = match aa.as_ref().unwrap().get(t_str) {
                        Some(re) => unsafe { (re as *const Regex).as_ref() },
                        _ => None,
                    };

                    let mul_line_re = match bb.as_ref().unwrap().get(t_str) {
                        Some(re) => unsafe { (re as *const (Regex, Regex)).as_ref() },
                        _ => None,
                    };

                    if single_line_re.is_some() || mul_line_re.is_some() {
                        files.push(File(path.to_path_buf(), single_line_re, mul_line_re));
                    }
                }
            }
        }
    } else {
        let aa = REGEX_TABLE.lock();
        let aa_guard = aa.as_ref().unwrap();

        let bb = REGEX_TABLE_MUL.lock();
        let bb_guard = bb.as_ref().unwrap();

        // 1. Try extension
        if let Some(t_str) = ext_str {
            let single_line_re = match aa_guard.get(t_str) {
                Some(re) => unsafe { (re as *const Regex).as_ref() },
                None => None,
            };

            let mul_line_re = match bb_guard.get(t_str) {
                Some(re) => unsafe { (re as *const (Regex, Regex)).as_ref() },
                None => None,
            };

            if single_line_re.is_some() || mul_line_re.is_some() {
                files.push(File(path.to_path_buf(), single_line_re, mul_line_re));
                return;
            }
        }

        // 2. Try filename (lowercase)
        if let Some(ref name) = file_name_str {
            let single_line_re = match aa_guard.get(name) {
                Some(re) => unsafe { (re as *const Regex).as_ref() },
                None => None,
            };

            let mul_line_re = match bb_guard.get(name) {
                Some(re) => unsafe { (re as *const (Regex, Regex)).as_ref() },
                None => None,
            };

            if single_line_re.is_some() || mul_line_re.is_some() {
                files.push(File(path.to_path_buf(), single_line_re, mul_line_re));
                return;
            }
        }

        // 3. Fallback for explicit targets
        if is_explicit {
            let re = unsafe {
                match (&*FALLBACK_REGEX as *const Regex).as_ref() {
                    Some(a) => a,
                    None => return,
                }
            };
            files.push(File(path.to_path_buf(), Some(re), None));
        }
    }
}

/// The status pass to filter_line
enum FilterLineStatus {
    /// default, without any previous status
    /// with the one line regex and the mutliline regex start and end
    None,

    /// in multiple line comment, everything is comment
    InMulLine,
}

struct FilterLiner<'this_file> {
    regex_single_line: Option<&'this_file Regex>,

    regex_multiple_line: Option<&'this_file (Regex, Regex)>,

    status: FilterLineStatus,
}

impl<'this_file> FilterLiner<'this_file> {
    fn filter_line(&mut self, line: &str, line_num: usize) -> Option<Crumb> {
        match self.status {
            FilterLineStatus::None => {
                // multi line first
                if let Some(aa) = self.regex_multiple_line {
                    match aa.0.find(line) {
                        Some(mat) => {
                            let position = mat.start();
                            let cap = aa.0.captures(line).unwrap();
                            let content = cap[2].to_string();
                            let comment_symbol_header = cap[1].to_string();
                            let mut res = if content.starts_with('!') {
                                Crumb::new(
                                    line_num,
                                    position,
                                    content,
                                    comment_symbol_header,
                                    String::new(),
                                )
                                .add_ignore_flag()
                            } else {
                                Crumb::new(
                                    line_num,
                                    position,
                                    content,
                                    comment_symbol_header,
                                    String::new(),
                                )
                            };

                            // crumb will have the tail
                            res.has_tail = true;

                            // update to in mul lines
                            self.status = FilterLineStatus::InMulLine;

                            return Some(res);
                        }
                        None => (),
                    }
                }

                if let Some(bb) = self.regex_single_line {
                    match bb.find(line) {
                        Some(mat) => {
                            let position = mat.start();
                            let cap = bb.captures(line).unwrap();
                            let content = cap[2].to_string();
                            let comment_symbol_header = cap[1].to_string();
                            let res = if content.starts_with('!') {
                                Crumb::new(
                                    line_num,
                                    position,
                                    content,
                                    comment_symbol_header,
                                    String::new(),
                                )
                                .add_ignore_flag()
                            } else {
                                Crumb::new(
                                    line_num,
                                    position,
                                    content,
                                    comment_symbol_header,
                                    String::new(),
                                )
                            };

                            return Some(res);
                        }
                        None => (),
                    }
                }
                return None;
            }
            FilterLineStatus::InMulLine => {
                let aa = self.regex_multiple_line.unwrap();
                if let Some(_mat) = aa.1.find(line) {
                    self.status = FilterLineStatus::None;
                    let cap = aa.1.captures(line).unwrap();
                    let raw_content = &cap[1];
                    let content = raw_content
                        .trim_start()
                        .trim_end_matches(['\r', '\n'])
                        .to_string();
                    let position = line.len() - line.trim_start().len();
                    let comment_symbol_endding = cap[2].to_string();
                    let cr = Crumb::new(
                        line_num,
                        position,
                        content,
                        String::new(),
                        comment_symbol_endding,
                    );
                    Some(cr)
                } else {
                    let content = line.trim_start().trim_end_matches(['\r', '\n']);
                    let position = line.len() - line.trim_start().len();
                    let mut cr = Crumb::new(
                        line_num,
                        position,
                        content.to_string(),
                        String::new(),
                        String::new(),
                    );
                    cr.has_tail = true;
                    Some(cr)
                }
            }
        }
    }
}

/// Operate this file
fn op_file(file: File, kwreg: &Option<Regex>, conf: Arc<RwLock<Config>>) -> Result<Option<Bread>> {
    let breads = match bake_bread(&file, kwreg, &conf.read().unwrap()) {
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
                delete_the_crumbs(bb)?;
                Ok(None)
            }
            None => Ok(None),
        }
    }
}

/// Make bread for this file
/// Major logic inside this function
fn bake_bread(file: &File, kwreg: &Option<Regex>, conf: &Config) -> Result<Option<Bread>> {
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
    let mut shadow_file = vec![]; // the copy of file for later range operation 

    // closure for keywords feature
    let mut keyword_checker_and_push = |mut cb: Crumb| {
        cb.has_tail = false;
        if kwreg.is_some() {
            // filter_keywords will update keyword even the crumb is ignored
            if cb.filter_keywords(kwreg.as_ref().unwrap()) {
                result.push(cb)
            }
        } else {
            if !cb.is_ignore() || conf.show_ignored {
                result.push(cb)
            }
        }
    };

    // make the new filter
    let mut fl = FilterLiner {
        regex_single_line: file.1,
        regex_multiple_line: file.2,
        status: FilterLineStatus::None,
    };

    loop {
        line_num += 1;
        match buf.read_line(&mut ss) {
            Ok(0) => {
                if head.is_some() {
                    keyword_checker_and_push(head.unwrap());
                }
                break;
            }
            Err(e) => {
                eprintln!(
                    "Warning: file {} had read error at line {}: {}",
                    file_p, line_num, e
                );
                if head.is_some() {
                    keyword_checker_and_push(head.unwrap());
                }
                break;
            }
            Ok(_) => match fl.filter_line(&ss, line_num) {
                Some(mut cb) => {
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

        if conf.range > 0 {
            shadow_file.push(ss.clone());
        }

        ss.clear()
    }

    // if range not equal 0, start to push the context around inside
    if conf.range > 0 {
        result.iter_mut().for_each(|crumb| {
            let ahead_ind = (crumb.line_num - 1).saturating_sub(conf.range as usize);
            let tail_ind = (crumb.line_num - 1)
                .saturating_add(conf.range as usize)
                .min(shadow_file.len());
            crumb.range_content = Some(
                (ahead_ind + 1..tail_ind + 1)
                    .zip(
                        shadow_file
                            .get(ahead_ind..tail_ind)
                            .map(|x| x.to_vec())
                            .unwrap(),
                    )
                    .collect(),
            );
        });
    }

    if result.len() == 0 {
        Ok(None)
    } else {
        Ok(Some(Bread::new(file_p, result)))
    }
}

/// delete crumbs and re-write the file
pub fn delete_the_crumbs(Bread { file_path, crumbs }: Bread) -> Result<String> {
    let all_delete_line_postion_pairs = crumbs
        .iter()
        .map(|crumb| crumb.all_lines_num_postion_pair())
        .flatten();

    delete_lines_on(&file_path, all_delete_line_postion_pairs)?;

    println!("deleted the crumbs in {}", file_path);
    Ok(file_path)
}

/// delete crumbs by special indexes
pub fn delete_the_crumbs_on_special_index(
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

    println!("deleted {} crumbs in {}", indexes.len(), file_path);

    Ok(file_path)
}

fn write_file_atomically(file_path: &str, lines: &[Vec<u8>]) -> Result<()> {
    let temp_path = format!("{}.tmp", file_path);
    let write_res = (|| {
        let mut temp_file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&temp_path)?;
        for line in lines {
            temp_file.write_all(line)?;
            temp_file.write_all(b"\n")?;
        }
        Ok(())
    })();

    if let Err(e) = write_res {
        let _ = fs::remove_file(&temp_path);
        return Err(e);
    }

    if let Err(e) = fs::rename(&temp_path, file_path) {
        let _ = fs::remove_file(&temp_path);
        return Err(e);
    }

    Ok(())
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
        .map(|line| line.into_bytes())
        .collect::<Vec<_>>();

    write_file_atomically(file_path, &finish_deleted)
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

/// restore the bread's crumb to normal comment
pub fn restore_the_crumb(Bread { file_path, crumbs }: Bread) -> Result<String> {
    let all_restore_lines = crumbs
        .iter()
        .map(|c| c.all_lines_num_postion_and_header_content())
        .flatten();

    restore_lines_on(&file_path, all_restore_lines)?;

    println!("restored the crumbs in {}", file_path);
    Ok(file_path)
}

/// restore the bread's crumb by special indexes
pub fn restore_the_crumb_on_special_index(
    Bread { file_path, crumbs }: Bread,
    indexes: HashSet<usize>,
) -> Result<String> {
    let mut all_restore_lines = Vec::with_capacity(indexes.len());
    for ind in &indexes {
        match crumbs.get(*ind) {
            Some(c) => all_restore_lines.append(&mut c.all_lines_num_postion_and_header_content()),
            None => return Err(io::Error::other("cannot find crumb index in bread")),
        }
    }

    restore_lines_on(&file_path, all_restore_lines.into_iter())?;

    println!("restored {} crumbs in {}", indexes.len(), file_path);
    Ok(file_path)
}

fn restore_lines_on<'a>(
    file_path: &'a str,
    all_restore_lines: impl Iterator<Item = (usize, usize, &'a str, &'a str, &'a str)>,
) -> Result<()> {
    let f = fs::File::open(&file_path)?;
    let reader = BufReader::new(f).lines();

    let mut table: HashMap<usize, (usize, &str, &str, &str)> =
        HashMap::with_capacity(all_restore_lines.size_hint().1.unwrap_or(0));

    all_restore_lines.for_each(|(line_num, pos, header, endding, content)| {
        table.insert(line_num, (pos, header, content, endding));
    });

    let mut new_file = Vec::with_capacity(reader.size_hint().1.unwrap_or(0));
    for (line_num, ll) in reader.enumerate() {
        if let Some((pos, header, content, endding)) = table.get(&(line_num + 1)) {
            let mut new_l = ll?;
            new_l.truncate(*pos);
            new_l.push_str(*header);
            if *header != "" {
                new_l.push_str(" ");
            }
            new_l.push_str(*content);
            new_l.push_str(*endding);

            new_file.push(new_l.into_bytes())
        } else {
            new_file.push(ll?.into_bytes());
        }
    }

    write_file_atomically(file_path, &new_file)
}

/// run format command with filepath input
pub fn run_format_command_to_file(
    fmt_command: &str,
    _files: impl IntoIterator<Item = String>,
) -> std::result::Result<(), String> {
    let mut command_splits = fmt_command.split(' ');
    let first = command_splits
        .next()
        .ok_or("fmt_command cannot be empty".to_string())?;

    let mut comm = Command::new(first);
    let mut child = comm
        .args(command_splits)
        .spawn()
        .expect("Cannot run the fmt_command");

    println!("running fmt command: {}", fmt_command);
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
    let threads_num: usize = thread::available_parallelism()
        .unwrap_or(THREAD_NUM.unwrap())
        .into();

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

    // #[test]
    // fn test_available_parallelism_on_my_machine() {
    //     dbg!(thread::available_parallelism().unwrap());
    // }
}
