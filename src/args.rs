use clap::Parser;
use std::ffi::OsString;

/// Command Line Args
#[derive(Default, Parser, Debug)]
#[clap(author = "ccQpein", version, about)]
pub struct Args {
    /// What are the filetypes you want to scan.
    #[clap(short, long)]
    pub(crate) filetypes: Vec<OsString>,

    /// The folder name should ignored
    #[clap(short = 'x', long = "ignore-dir")]
    pub(crate) ignore_dirs: Vec<OsString>,

    /// Keywords
    #[clap(short, long)]
    pub(crate) keywords: Option<Vec<String>>,

    /// Expand dictionary json file path
    #[clap(short, long)]
    pub(crate) jsonx: Option<String>,

    /// files/dirs input directly
    #[clap(name = "files/dirs", default_value = ".")]
    pub(crate) targets: Vec<String>,

    /// delete the crumbs
    #[clap(short = 'D', long = "del")]
    pub(crate) delete: bool,

    /// format command after delete crumbs
    #[clap(long = "fmt")]
    pub(crate) fmt_command: Option<String>,
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

        if other.fmt_command.is_some() {
            self.fmt_command = other.fmt_command
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
    }

    /// fmt command is the shell command, so it has to be string
    #[test]
    fn test_parse_the_fmt_string() {
        let args = vec!["codeitlater", "--fmt", "aaa bbb"];
        assert_eq!(Args::parse_from(args).fmt_command.unwrap(), "aaa bbb")
    }
}
