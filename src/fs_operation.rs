use super::config::{Config, KEYWORDS_REGEX, REGEX_TABLE};
use regex::Regex;
use std::fmt;
use std::fs::read_dir;
use std::io::prelude::*;
use std::{io::Result, path::Path, path::PathBuf, thread};

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

type Dirs = Vec<PathBuf>;

#[derive(Debug)]
struct File(PathBuf, &'static Regex);

impl File {
    fn to_string(&self) -> String {
        self.0.as_os_str().to_os_string().into_string().unwrap()
    }
}

type Files = Vec<File>;

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
                            let re = unsafe { (re as *const Regex).clone().as_ref().unwrap() };
                            f.push(File(path, re))
                        }
                    }
                }
            } else {
                if let Some(t) = path.extension() {
                    // file has extension
                    let aa = REGEX_TABLE.lock();
                    if let Some(re) = aa.as_ref().unwrap().get(t.to_str().unwrap()) {
                        // and has regex for this type
                        let re = unsafe { (re as *const Regex).clone().as_ref().unwrap() };
                        f.push(File(path, re))
                    }
                }
            }
        }
    }
    Ok((f, d))
}

#[derive(Debug)]
struct Bread {
    file_path: String,
    crumbs: Vec<Crumb>,
}

impl Bread {
    fn new(f: String, crumbs: Vec<Crumb>) -> Self {
        Bread {
            file_path: f,
            crumbs,
        }
    }
}

impl fmt::Display for Bread {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "|-- {}\n", self.file_path)?;
        for c in &self.crumbs {
            write!(f, "  |-- {}\n", c)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
struct Crumb {
    line_num: usize,
    content: String,
}

impl Crumb {
    fn has_keywords(&self, re: &Regex) -> bool {
        re.is_match(&self.content)
    }
}

impl fmt::Display for Crumb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Line {}: {}", self.line_num, self.content)?;
        Ok(())
    }
}

fn filter_line(line: &str, line_num: usize, re: &Regex) -> Option<Crumb> {
    match re.captures(line) {
        Some(content) => Some(Crumb {
            line_num,
            content: content[2].to_string(),
        }),
        None => None,
    }
}

fn op_file(file: File, kwreg: &Option<Regex>) -> Result<Option<Bread>> {
    // start to read file
    let mut buf = vec![];
    let file_p = file.to_string();
    let mut f: std::fs::File = std::fs::File::open(file.0)?;
    f.read_to_end(&mut buf)?;

    let mut line_num = 0;
    let mut ss = String::new();
    let mut buf = buf.as_slice();
    let mut result = vec![];
    loop {
        line_num += 1;
        match buf.read_line(&mut ss) {
            Ok(0) | Err(_) => break, // if EOF or any error in this file, break
            Ok(_) => match filter_line(&ss, line_num, file.1) {
                Some(cb) => {
                    if kwreg.is_some() {
                        if cb.has_keywords(kwreg.as_ref().unwrap()) {
                            result.push(cb)
                        }
                    } else {
                        result.push(cb)
                    }
                }
                None => (),
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

pub fn handle_files(conf: &Config) {
    let mut all_files: Vec<File> = conf
        .dirs
        .iter()
        .map(|d| all_files_in_dir(d, conf).unwrap())
        .flatten()
        .collect();

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
        .for_each(|b| println!("{}", b));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_files_and_dirs_in_path() -> Result<()> {
        let (fs, dirs) = files_and_dirs_in_path("./tests", &Default::default())?;

        assert_eq!(dirs.len(), 0);
        assert_eq!(fs[0].0, PathBuf::from("./tests/test.rs"),);
        Ok(())
    }
}
