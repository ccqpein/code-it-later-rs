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
    pub(crate) content: String,

    ignore: bool,
}

impl Crumb {
    /// side effect: will change keyword to Some(_) if match successed
    pub fn filter_keywords(&mut self, re: &Regex) -> bool {
        match re.captures(&self.content) {
            Some(a) => {
                self.keyword = Some(a[1].to_string());
                self.content = a[2].to_string();
                true
            }
            None => false,
        }
    }

    pub fn has_tail(&self) -> bool {
        self.content.ends_with("...")
    }

    /// add tail crumbs in this one
    pub fn add_tail(&mut self, tail: Self) {
        // update the first crumb's content
        self.content = self.content.trim_end().trim_end_matches("...").to_string();
        self.content.push(' ');
        self.content.push_str(&tail.content);
        self.tails.push(tail);
    }

    pub fn new(line_num: usize, position: usize, keyword: Option<String>, content: String) -> Self {
        Self {
            line_num,
            position,
            keyword,
            tails: vec![],
            content,
            ignore: false,
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
        format!("{}: {}{}", self.line_num, kw, self.content)
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
        write!(f, "Line {}: {}{}\n", self.line_num, a, self.content)?;
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
        assert_eq!(a.content, "test1");

        // test 2
        let mut a: Crumb = Default::default();
        a.content = "test1".to_string();

        assert!(!a.filter_keywords(&Regex::new(&format!("({}):\\s*(.*)", "TODO")).unwrap()));
        assert_eq!(a.keyword, None);

        // test 3
        let mut a: Crumb = Default::default();
        a.content = "!TODO: test3".to_string();
        a.ignore = true;
        dbg!(&a);
        assert!(
            a.filter_keywords(&Regex::new(&format!("({}|{}):\\s*(.*)", "TODO", "MARK")).unwrap())
        );
        dbg!(&a);
        assert_eq!(a.keyword, Some("TODO".to_string()));
    }

    #[test]
    fn test_to_org() {
        let b0 = Bread::new("a".into(), vec![]);
        assert_eq!(b0.to_org().unwrap(), "* a\n".to_string());

        let b1 = Bread::new(
            "a".into(),
            vec![
                Crumb::new(1, 0, None, "1".to_string()),
                Crumb::new(2, 0, Some("TODO".to_string()), "2".to_string()),
            ],
        );
        assert_eq!(b1.to_org().unwrap(), "* a\n** TODO 2\n".to_string());
    }
}
