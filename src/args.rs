//! The arguments of codeitlater are using

use clap::Parser;
use std::{
    ffi::OsString,
    fs::File,
    io::{BufRead, BufReader},
};

/// Command Line Args
#[derive(Default, Parser, Debug)]
#[command(author = "ccQpein", version, about)]
pub struct Args {
    /// What are the filetypes you want to scan.
    #[arg(short, long)]
    pub(crate) filetypes: Vec<OsString>,

    /// The folder name should ignored
    #[arg(short = 'x', long = "ignore-dir")]
    pub(crate) ignore_dirs: Vec<OsString>,

    /// Keywords
    #[arg(short, long)]
    pub(crate) keywords: Option<Vec<String>>,

    /// Expand dictionary json file path
    #[arg(short, long)]
    pub(crate) jsonx: Option<String>,

    /// Files/Dirs input directly
    #[arg(value_name = "files/dirs", default_value = ".")]
    pub(crate) targets: Vec<String>,

    /// Delete the crumbs
    #[arg(short = 'D', long = "del")]
    pub(crate) delete: bool,

    /// Restore the crumbs back to normal comment
    #[arg(short = 'R', long = "restore")]
    pub(crate) restore: bool,

    /// Format command after delete crumbs
    #[arg(long = "fmt")]
    pub(crate) fmt_command: Option<String>,

    /// Output format: json, list
    #[arg(short = 'O', long = "output-format")]
    pub(crate) output_format: Option<String>,

    /// Show all ignored crumbs
    #[arg(long = "show-ignored", default_value = "false")]
    pub(crate) show_ignore: bool,

    /// Config file location, default value it "."
    #[arg(short = 'C', long = "config", default_value = ".")]
    pub(crate) config_location: String,
}

impl Args {
    /// cover this args with other, self values totally rewrotten by other
    /// if both of args have same fields. Except ignore dirs, they are merged
    pub fn cover(&mut self, mut other: Self) {
        if other.filetypes.len() != 0 {
            self.filetypes = other.filetypes
        }

        if other.ignore_dirs.len() != 0 {
            self.ignore_dirs.append(&mut other.ignore_dirs)
        }

        if other.keywords.is_some() {
            self.keywords = other.keywords
        }

        if other.jsonx.is_some() {
            self.jsonx = other.jsonx
        }

        if other.targets.len() != 0 {
            self.targets = other.targets;
        }

        if other.delete {
            self.delete = other.delete
        }

        if other.restore {
            self.restore = other.restore
        }

        if other.fmt_command.is_some() {
            self.fmt_command = other.fmt_command
        }

        if other.output_format.is_some() {
            self.output_format = other.output_format
        }

        self.show_ignore = other.show_ignore
    }

    pub fn fmt_command(&self) -> Option<&String> {
        self.fmt_command.as_ref()
    }

    pub fn config_location(&self) -> String {
        self.config_location.to_string()
    }
}

fn split_space_exclude_those_in_inner_string(s: &str) -> Result<Vec<String>, String> {
    let mut result = vec![];
    let mut buf = vec![];
    let mut in_string = false;

    for b in s.bytes() {
        if b == b' ' && !in_string {
            if !buf.is_empty() {
                result.push(String::from_utf8(buf).map_err(|e| e.to_string())?);
                buf = vec![];
            }
        } else {
            if b == b'"' {
                in_string ^= true;
                continue;
            }
            buf.push(b);
        }
    }

    if !buf.is_empty() {
        result.push(String::from_utf8(buf).map_err(|e| e.to_string())?);
    }

    Ok(result)
}

fn read_config_raw_content<R: BufRead>(content: R) -> Vec<String> {
    let buf_reader = BufReader::new(content);
    let mut a = vec!["codeitlater".to_string()];
    a.append(
        &mut buf_reader
            .lines()
            .filter_map(|l| {
                let ll = l.unwrap();
                if ll.is_empty() {
                    None
                } else {
                    Some(split_space_exclude_those_in_inner_string(&ll).unwrap())
                }
            })
            .flatten()
            .collect::<Vec<String>>(),
    );
    a
}

pub fn parse_from_current_path_config(config_folder: String) -> Option<Args> {
    match File::open(config_folder + "/.codeitlater") {
        Ok(f) => Some(Args::parse_from(read_config_raw_content(BufReader::new(f)))),
        Err(_e) => {
            //println!("{}", _e.to_string());
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_parse_from_iter() {
        let args = vec!["codeitlater", "-x", "dd"];
        assert_eq!(Args::parse_from(args).ignore_dirs, vec!["dd"]);

        let args = vec!["codeitlater", "-x", "dd", "-x", "ff"];
        assert_eq!(Args::parse_from(args).ignore_dirs, vec!["dd", "ff"]);

        // if there are some options, "--" is required
        let args = vec!["codeitlater", "-x", "dd", "--", "a", "b", "c"];
        assert_eq!(
            Args::parse_from(args).targets,
            vec!["a".to_string(), "b".to_string(), "c".to_string()]
        );

        let args = vec!["codeitlater", "--", "a", "b", "c"];
        assert_eq!(
            Args::parse_from(args).targets,
            vec!["a".to_string(), "b".to_string(), "c".to_string()]
        );

        let args = vec!["codeitlater", "a", "b", "c"];
        assert_eq!(
            Args::parse_from(args).targets,
            vec!["a".to_string(), "b".to_string(), "c".to_string()]
        );

        let args = vec!["codeitlater", "--del", "--", "a", "b", "c"];
        assert_eq!(
            Args::parse_from(args).targets,
            vec!["a".to_string(), "b".to_string(), "c".to_string()]
        );

        let args = vec!["codeitlater", "-x", "dd", "-x", "ff", "-D", "a", "b", "c"];
        let args = Args::parse_from(args);
        assert_eq!(
            args.targets,
            vec!["a".to_string(), "b".to_string(), "c".to_string()]
        );
        assert_eq!(args.delete, true);
        assert_eq!(args.ignore_dirs, vec!["dd", "ff"]);

        let args = vec![
            "codeitlater",
            "-x",
            "dd",
            "-x",
            "ff",
            "-D",
            "a",
            "b",
            "c",
            "-R",
        ];
        let args = Args::parse_from(args);
        assert_eq!(
            args.targets,
            vec!["a".to_string(), "b".to_string(), "c".to_string()]
        );
        assert_eq!(args.delete, true);
        assert_eq!(args.ignore_dirs, vec!["dd", "ff"]);
        assert_eq!(args.restore, true);
    }

    #[test]
    fn test_read_current_path_config() {
        let content = "
-x target

-k    TODO"
            .as_bytes();
        //dbg!(read_config_raw_content(content));
        assert_eq!(
            vec!["codeitlater", "-x", "target", "-k", "TODO"],
            read_config_raw_content(content)
        );
    }

    /// fmt command is the shell command, so it has to be string
    #[test]
    fn test_parse_the_fmt_string() {
        let args = vec!["codeitlater", "--fmt", "aaa bbb"];
        assert_eq!(Args::parse_from(args).fmt_command.unwrap(), "aaa bbb");

        let args = vec!["codeitlater", "--fmt", r#""cargo fmt""#];
        assert_eq!(
            Args::parse_from(args).fmt_command.unwrap(),
            r#""cargo fmt""#
        )
    }
}
