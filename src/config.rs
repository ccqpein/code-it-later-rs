use regex::{Regex, RegexBuilder};
use std::collections::HashMap;
use std::ffi::OsString;

use std::fs::File;
use std::io::Read;
use std::sync::{LazyLock, Mutex};

use super::args::Args;

/// Inner dictionary
const DICT: &'static str = r#"
{
"rs":["//", "/\\*"],
"go":["//", "/\\*", "// "],
"lisp":[";"],
"asd":[";"],
"asdf":[";"],
"py":["\\#"],
"hs":["-- "],
"el":[";"],
"clj":[";"],
"js":["//"]
}
"#;

static TABLE: LazyLock<Mutex<HashMap<String, Vec<String>>>> =
    LazyLock::new(|| Mutex::new(serde_json::from_str(DICT).unwrap()));

pub static REGEX_TABLE: LazyLock<Mutex<HashMap<String, Regex>>> = LazyLock::new(|| {
    Mutex::new({
        let a = TABLE.lock().unwrap();
        a.iter()
            .map(|(k, v)| (k.clone(), Regex::new(&make_regex(v)).unwrap()))
            .collect()
    })
});

pub static KEYWORDS_REGEX: LazyLock<Mutex<Option<Regex>>> = LazyLock::new(|| Mutex::new(None));

/// Update static table with new raw_json str
fn update_table(raw_json: &str) {
    let new_table: HashMap<String, Vec<String>> = serde_json::from_str(raw_json).unwrap();

    let mut table = TABLE.lock().unwrap();
    for (k, v) in new_table.iter() {
        table.insert(k.clone(), v.clone());
    }

    let mut re_table = REGEX_TABLE.lock().unwrap();
    table
        .iter()
        .map(|(k, v)| (k.clone(), Regex::new(&make_regex(v)).unwrap()))
        .for_each(|(k, v)| {
            let _ = re_table.insert(k, v);
        });
}

/// Making regex string
fn make_regex(com_syms: &Vec<String>) -> String {
    let mut head = String::new();
    for s in com_syms {
        head.push('|');
        head.push_str(s);
        head.push_str("+");
    }

    let _ = head.drain(..1).collect::<String>();

    format!("({}):=\\s+(.*)", head)
}

/// making the keyword regex, case insensitive
pub(super) fn make_key_regex(keywords: &Vec<String>) {
    let mut ss = String::new();
    for s in keywords {
        ss.push_str(&s);
        ss.push('|');
    }

    let _ = ss.drain(ss.len() - 1..).collect::<String>();
    let mut kk = KEYWORDS_REGEX.lock().unwrap();
    *kk = Some(
        RegexBuilder::new(&format!("({}):\\s*(.*)", ss))
            .case_insensitive(true)
            .build()
            .unwrap(),
    );
}

pub fn clean_keywords_table() {
    let mut kk = KEYWORDS_REGEX.lock().unwrap();
    *kk = None;
}

#[derive(Clone, Debug)]
pub(super) enum OutputFormat {
    None,
    Json,
    List,
}

impl Default for OutputFormat {
    fn default() -> Self {
        Self::None
    }
}

/// config when running
#[derive(Default, Debug, Clone)]
pub struct Config {
    pub(super) filetypes: Vec<OsString>,
    pub(super) ignore_dirs: Vec<OsString>,
    pub(super) files: Vec<String>,

    /// if delete
    pub(super) delete: bool,

    /// if restore
    pub(super) restore: bool,

    /// output format
    pub(super) output: OutputFormat,

    /// show ignored
    pub(super) show_ignored: bool,
}

impl From<&Args> for Config {
    fn from(a: &Args) -> Self {
        match &a.jsonx {
            Some(j) => {
                let mut buf = vec![];
                File::open(j).unwrap().read_to_end(&mut buf).unwrap();
                update_table(&String::from_utf8(buf).unwrap());
            }
            None => (),
        }

        match &a.keywords {
            Some(kk) => make_key_regex(&kk),
            None => (),
        }

        let output = match &a.output_format {
            Some(v) if v.to_lowercase().as_str() == "json" => OutputFormat::Json,
            Some(v) if v.to_lowercase().as_str() == "list" => OutputFormat::List,
            _ => OutputFormat::None,
        };

        Self {
            filetypes: a.filetypes.clone(),
            ignore_dirs: a.ignore_dirs.clone(),
            files: a.targets.clone(),

            delete: a.delete,
            // delete and restore cannot be true at the same time
            // and delete has higher priority
            restore: if a.delete { false } else { a.restore },

            output,
            show_ignored: a.show_ignore,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;

    #[test]
    fn test_update_table() {
        assert_eq!(
            TABLE.lock().unwrap().get("rs").unwrap(),
            &vec![String::from("//"), String::from(r#"/\*"#)]
        );

        assert_eq!(
            REGEX_TABLE.lock().unwrap().get("rs").unwrap().as_str(),
            &String::from(r#"(//+|/\*+):=\s+(.*)"#)
        );

        // update here
        update_table(r##"{"rs":["//","#"]}"##);

        assert_eq!(
            TABLE.lock().unwrap().get("rs").unwrap(),
            &vec![String::from("//"), String::from("#")]
        );

        assert_eq!(
            REGEX_TABLE.lock().unwrap().get("rs").unwrap().as_str(),
            &String::from(r#"(//+|#+):=\s+(.*)"#)
        );

        // more test
        update_table(r#"{"rs":["//","/\\*"]}"#);

        assert_eq!(
            TABLE.lock().unwrap().get("rs").unwrap(),
            &vec![String::from("//"), String::from("/\\*")]
        );

        assert_eq!(
            TABLE.lock().unwrap().get("rs").unwrap(),
            &vec![String::from("//"), String::from(r#"/\*"#)]
        );

        assert_eq!(
            REGEX_TABLE.lock().unwrap().get("rs").unwrap().as_str(),
            &String::from(r#"(//+|/\*+):=\s+(.*)"#)
        );
    }

    #[test]
    fn test_update_table_with_json() {
        let mut buf = vec![];
        File::open("./tests/testcases/test.json")
            .unwrap()
            .read_to_end(&mut buf)
            .unwrap();
        let ss: &str = &String::from_utf8(buf).unwrap();

        // This means file content equal the pure str
        // which already test before in test_update_table
        assert_eq!(
            serde_json::from_str::<HashMap<String, Vec<String>>>(ss).unwrap(),
            serde_json::from_str(r#"{"rs":["//","/\\*"]}"#).unwrap() // json reader need one more '/'
        );
    }

    #[test]
    fn test_make_regex() {
        assert_eq!(
            make_regex(&vec![String::from("//"), String::from(";")]),
            String::from(r#"(//+|;+):=\s+(.*)"#)
        );

        assert_eq!(
            make_regex(&vec![String::from("//"), String::from(r#"/\*"#)]),
            String::from(r#"(//+|/\*+):=\s+(.*)"#)
        );
    }

    #[test]
    fn test_regex() {
        let re = Regex::new(&make_regex(&vec![String::from("--"), String::from(";")])).unwrap();
        let cap = re.captures("Aabbcc --:= test").unwrap();
        assert_eq!(&cap[2], "test");

        let cap = re.captures("Aabbcc ;:= test").unwrap();
        assert_eq!(&cap[2], "test");

        let cap = re.captures("Aabbcc ;;;:= test").unwrap();
        assert_eq!(&cap[2], "test");
        assert_eq!(&cap[1], ";;;");

        assert!(re.captures("Aabbcc #:= test").is_none());

        assert!(re.captures("Aabbcc ; test").is_none());

        assert!(re.captures("Aabbcc ; := test").is_none());

        // more tests
        let re = Regex::new(&make_regex(&vec![
            String::from("//"),
            String::from(r#"/\*"#),
            String::from(r#"// "#),
        ]))
        .unwrap();
        assert!(re.captures("err := test").is_none());
        assert!(re.captures("err // := test").is_some());
        assert_eq!(&re.captures("err // := test").unwrap()[1], "// ");
    }

    #[test]
    fn test_restore_overwrited_by_delete() {
        let mut arg: Args = Default::default();
        arg.delete = true;
        arg.restore = true;
        let conf = Config::from(&arg);
        assert!(conf.delete);
        assert!(!conf.restore);

        arg.delete = false;
        arg.restore = true;
        let conf = Config::from(&arg);
        assert!(!conf.delete);
        assert!(conf.restore);
    }
}
