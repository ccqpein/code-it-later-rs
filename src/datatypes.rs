use regex::Regex;
use std::fmt;

#[derive(Debug, PartialEq, Eq)]
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
            write!(f, "  |-- {}\n", c)?;
        }
        Ok(())
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Crumb {
    pub(crate) line_num: usize,
    /// store tail lines' numbers after `line_num`
    tails_line_num: Vec<usize>,
    pub(crate) keyword: Option<String>,
    pub(crate) content: String,
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

    pub fn add_tail(&mut self, tail: Self) {
        self.content = self.content.trim_end().trim_end_matches("...").to_string();
        self.content.push(' ');
        self.content.push_str(&tail.content);
        self.tails_line_num.push(tail.line_num);
    }

    pub fn new(line_num: usize, keyword: Option<String>, content: String) -> Self {
        Self {
            line_num,
            keyword,
            tails_line_num: vec![],
            content,
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
        a.append(&mut self.tails_line_num.clone());
        a
    }
}

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
        write!(f, "Line {}: {}{}", self.line_num, a, self.content)?;
        Ok(())
    }
}

pub trait InteractShow {}

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
    }

    #[test]
    fn test_to_org() {
        let b0 = Bread::new("a".into(), vec![]);
        assert_eq!(b0.to_org().unwrap(), "* a\n".to_string());

        let b1 = Bread::new(
            "a".into(),
            vec![
                Crumb::new(1, None, "1".to_string()),
                Crumb::new(2, Some("TODO".to_string()), "2".to_string()),
            ],
        );
        assert_eq!(b1.to_org().unwrap(), "* a\n** TODO 2\n".to_string());
    }
}
