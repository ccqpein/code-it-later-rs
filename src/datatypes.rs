use regex::Regex;
use serde::Serialize;
use std::fmt;

/// major data struct including file path and all crumbs
#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct Bread {
    pub(super) file_path: String,
    pub(super) crumbs: Vec<Crumb>,
}

impl Bread {
    pub fn new(f: String, crumbs: Vec<Crumb>) -> Self {
        Bread {
            file_path: f,
            crumbs,
        }
    }

    pub fn to_org(&self) -> Result<String, !> {
        let mut content = format!("* {}\n", self.file_path);
        self.crumbs
            .iter()
            .filter_map(|c| c.to_org())
            .for_each(|org_inside| {
                content += "** ";
                content += &org_inside;
                content += "\n"
            });
        Ok(content)
    }
}

impl fmt::Display for Bread {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "|-- {}\n", self.file_path)?; // write file_path

        for c in &self.crumbs {
            write!(f, "  |-- {}", c)?;
        }
        Ok(())
    }
}

/// Crumb including the data of this line
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize)]
pub struct Crumb {
    pub(crate) line_num: usize,

    #[serde(skip)]
    /// the position of the crumb start from in this line
    pub(crate) position: usize,

    /// store tail lines' numbers after `line_num`
    tails: Vec<Crumb>,

    pub(crate) keyword: Option<String>,

    /// view_content use to print out
    /// like in tails and keywords
    /// the `content` below keep original content
    pub(crate) view_content: String,

    /// the content including original content after :=
    /// maybe different than view_content
    pub(crate) content: String,

    /// record the crumb header for restore
    /// like in lisp `;;;:= here`, `;;;` should be header
    comment_symbol_header: String,

    /// ignore this crumb or not
    ignore: bool,

    /// range content
    pub(crate) range_content: Option<Vec<(usize, String)>>,
}

impl Crumb {
    pub fn new_for_test(
        line_num: usize,
        position: usize,
        tails: Vec<Crumb>,
        keyword: Option<String>,
        view_content: String,
        content: String,
        comment_symbol_header: String,
        ignore: bool,
    ) -> Self {
        Self {
            line_num,
            position,
            tails,
            keyword,
            view_content,
            content,
            comment_symbol_header,
            ignore,
            range_content: None,
        }
    }

    /// side effect: will change keyword to Some(_) if match successed
    pub fn filter_keywords(&mut self, re: &Regex) -> bool {
        match re.captures(&self.content) {
            Some(a) => {
                self.keyword = Some(a[1].to_string());
                self.view_content = a[2].to_string();
                true
            }
            None => false,
        }
    }

    pub fn has_tail(&self) -> bool {
        self.view_content.ends_with("...")
    }

    /// add tail crumbs in this one
    pub fn add_tail(&mut self, tail: Self) {
        // update the first crumb's content
        self.view_content = self
            .view_content
            .trim_end()
            .trim_end_matches("...")
            .to_string();
        self.view_content.push(' ');
        self.view_content.push_str(&tail.content);
        self.tails.push(tail);
    }

    pub fn new(
        line_num: usize,
        position: usize,
        content: String,
        comment_symbol_header: String,
    ) -> Self {
        Self {
            line_num,
            position,
            keyword: None,
            tails: vec![],
            view_content: content.clone(),
            content,
            comment_symbol_header,
            ignore: false,
            range_content: None,
        }
    }

    /// keyword crumb can transfer to org string
    pub fn to_org(&self) -> Option<String> {
        match &self.keyword {
            Some(k) => Some(format!("{} {}", k, self.content)),
            None => None,
        }
    }

    /// return this crumb line_num and all tails line numbers if it has tails
    pub fn all_lines_num(&self) -> Vec<usize> {
        let mut a = vec![self.line_num];
        a.append(&mut self.tails.iter().map(|t| t.line_num).collect());
        a
    }

    /// return this crumb line numbers and the position of lines pairs
    pub fn all_lines_num_postion_pair(&self) -> Vec<(usize, usize)> {
        let mut a = vec![(self.line_num, self.position)];
        a.append(
            &mut self
                .tails
                .iter()
                .map(|t| (t.line_num, t.position))
                .collect(),
        );
        a
    }

    /// return this crumb line numbers, the position, the header and content of lines pairs
    pub fn all_lines_num_postion_and_header_content(&self) -> Vec<(usize, usize, &str, &str)> {
        let mut a = vec![(
            self.line_num,
            self.position,
            self.comment_symbol_header.as_str(),
            self.content.as_str(),
        )];
        a.append(
            &mut self
                .tails
                .iter()
                .map(|t| {
                    (
                        t.line_num,
                        t.position,
                        self.comment_symbol_header.as_str(),
                        t.content.as_str(),
                    )
                })
                .collect(),
        );
        a
    }

    // add the ignore flag to this crumb
    pub fn add_ignore_flag(mut self) -> Self {
        self.ignore = true;
        self
    }

    pub fn is_ignore(&self) -> bool {
        self.ignore
    }

    pub fn list_format(&self) -> String {
        let kw = match self.keyword {
            Some(ref k) => {
                let mut c = String::from(k);
                c.push_str(": ");
                c
            }
            None => "".to_string(),
        };
        format!("{}: {}{}", self.line_num, kw, self.view_content)
    }

    pub fn range_format(&self) -> String {
        match &self.range_content {
            Some(content) => content
                .iter()
                .map(|(ln, line)| format!("Line {}: {}", ln, line))
                .collect::<Vec<_>>()
                .join("\n"),
            None => String::new(),
        }
    }
}

/// default format
impl fmt::Display for Crumb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let a = match self.keyword {
            Some(ref k) => {
                let mut c = String::from(k);
                c.push_str(": ");
                c
            }
            None => "".to_string(),
        };
        write!(f, "Line {}: {}{}\n", self.line_num, a, self.view_content)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_keyowrds() {
        let mut a: Crumb = Default::default();
        a.content = "TODO: test1".to_string();

        assert!(a.filter_keywords(&Regex::new(&format!("({}):\\s*(.*)", "TODO")).unwrap()));
        assert_eq!(a.keyword, Some("TODO".to_string()));

        a.content = "TODO: test1".to_string();
        assert!(
            a.filter_keywords(&Regex::new(&format!("({}|{}):\\s*(.*)", "TODO", "MARK")).unwrap())
        );
        assert_eq!(a.keyword, Some("TODO".to_string()));
        assert_eq!(a.view_content, "test1");

        // test 2
        let mut a: Crumb = Default::default();
        a.content = "test1".to_string();

        assert!(!a.filter_keywords(&Regex::new(&format!("({}):\\s*(.*)", "TODO")).unwrap()));
        assert_eq!(a.keyword, None);

        // test 3
        let mut a: Crumb = Default::default();
        a.content = "!TODO: test3".to_string();
        a.ignore = true;
        //dbg!(&a);
        assert!(
            a.filter_keywords(&Regex::new(&format!("({}|{}):\\s*(.*)", "TODO", "MARK")).unwrap())
        );
        //dbg!(&a);
        assert_eq!(a.keyword, Some("TODO".to_string()));
    }
}
