use lazy_static::*;
use regex::{Regex, RegexBuilder};
use std::collections::HashMap;
use std::ffi::OsString;

use std::fs::File;
use std::io::Read;
use std::sync::Mutex;

use super::args::Args;

/// Inner dictionary
const DICT: &'static str = r#"
{
"rs":["//", "/\\*"],
"go":["//", "/\\*", "// "],
"lisp":[";"],
"py":["\\#"],
"hs":["-- "],
"el":[";"],
"clj":[";"],
"js":["//"]
}
"#;

lazy_static! {
    /// The table including all languages filetype and its comment symbols
    static ref TABLE: Mutex<HashMap<String, Vec<String>>> =
        Mutex::new(serde_json::from_str(DICT).unwrap());

    /// Regex table, like "rs" => "(//+):=\s+(.*)"
    pub static ref REGEX_TABLE: Mutex<HashMap<String, Regex>> = Mutex::new({
        let a = TABLE.lock().unwrap();
        a.iter().map(|(k, v)| (k.clone(), Regex::new(&make_regex(v)).unwrap())).collect()
    });

    pub static ref KEYWORDS_REGEX: Mutex<Option<Regex>> = Mutex::new(None);
}

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
            Some(v) if v == "json" => OutputFormat::Json,
            _ => OutputFormat::None,
        };

        Self {
            filetypes: a.filetypes.clone(),
            ignore_dirs: a.ignore_dirs.clone(),
            files: a.targets.clone(),

            delete: a.delete,
            output,
            show_ignored: a.show_ignore.unwrap_or(false),
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
    }
}
