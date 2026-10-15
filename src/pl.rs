//! support clean data structures for plist.

use std::fmt;
use std::io;
use std::path::Path;

/// 2 spaces per nesting level
const INDENT: &str = "  ";

fn indent(f: &mut fmt::Formatter<'_>, depth: usize) -> fmt::Result {
    for _ in 0..depth {
        f.write_str(INDENT)?;
    }
    Ok(())
}

/// simplify the creation of a plist file with `PListTopLevel`
pub struct PListFile(PListTopLevel);

impl PListFile {
    /// creates a `PListTopLevel` with starting template.
    /// Use `PListTopLevel` if you don't want abstraction- rather building top-level yourself.
    pub fn new(pl_ver: &str) -> Self {
        let doc = PListTopLevel::new()
            .head(PList::VeryConfused(Tag::new(
                "xml".into(),
                vec![
                    ("version".into(), "1.0".into()),
                    ("encoding".into(), "UTF-8".into()),
                ],
                vec![],
            )))
            .head(PList::Angry(Tag::new(
                "DOCTYPE".into(),
                vec![],
                vec![
                    "plist".into(),
                    "PUBLIC".into(),
                    "\"-//Apple//DTD PLIST 1.0//EN\"".into(),
                    "\"http://www.apple.com/DTDs/PropertyList-1.0.dtd\"".into(),
                ],
            )))
            .body(PList::Normal(
                Tag::new(
                    "plist".into(),
                    vec![("version".into(), pl_ver.to_string())],
                    vec![],
                ),
                vec![],
            ));

        PListFile(doc)
    }

    /// hands back the top level so nodes can be added to the plist body.
    pub fn top(&mut self) -> &mut PListTopLevel {
        &mut self.0
    }

    /// unwraps into the `PListTopLevel` if you'd rather build it yourself.
    pub fn into_top(self) -> PListTopLevel {
        self.0
    }

    /// fills the root `<plist>` element with its content
    /// (usually a `dict!` or `array!` node).
    pub fn content(mut self, root: impl Into<PList>) -> Self {
        let root = root.into();
        let nodes = &mut self.0.0;
        let idx = match nodes
            .iter()
            .rposition(|n| matches!(n, PList::Normal(t, _) if t.name() == "plist"))
        {
            Some(i) => i,
            None => nodes.len().saturating_sub(1),
        };
        if let Some(PList::Normal(_, kids)) = nodes.get_mut(idx) {
            *kids = vec![root];
        }
        self
    }

    /// serializes the whole document into the file at `path`.
    pub fn write(&self, path: impl AsRef<Path>) -> io::Result<()> {
        std::fs::write(path, self.to_string())
    }
}

impl fmt::Display for PListFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

pub struct PListTopLevel(Vec<PList>);

impl PListTopLevel {
    pub fn new() -> Self {
        PListTopLevel(vec![])
    }

    pub fn head(mut self, item: PList) -> Self {
        match item {
            PList::Value(_) | PList::Normal(_, _) => {
                panic!("unable to express in top level of XML")
            }
            _ => {
                self.0.push(item);
                self
            }
        }
    }

    pub fn body(mut self, item: PList) -> Self {
        self.0.push(item);
        self
    }
}

impl PListTopLevel {
    pub fn remove(mut self, idx: usize) -> Self {
        self.0.remove(idx);
        self
    }

    /// non-consuming sibling of [`Self::body`]
    pub fn push(&mut self, item: PList) {
        self.0.push(item);
    }

    pub fn get_mut(&mut self, idx: usize) -> Option<&mut PList> {
        self.0.get_mut(idx)
    }
}

impl Default for PListTopLevel {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for PListTopLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (idx, node) in self.0.iter().enumerate() {
            if idx > 0 {
                f.write_str("\n")?;
            }
            node.fmt_at(f, 0)?;
        }
        Ok(())
    }
}

/// Means can be `<name x="2430">` or `<name class>` or `<name>`.
#[derive(Clone)]
pub struct Tag(String, Vec<(String, String)>, Vec<String>);

impl Tag {
    pub fn new(name: String, assigns: Vec<(String, String)>, classes: Vec<String>) -> Self {
        Self(name, assigns, classes)
    }

    pub fn name(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Tag {
    /// simple wrapper to just make as a string the whole blob to be evaluated
    /// e.g `Tag("e", vec![("w", "s")], vec!["gay"])` -> `"e w=s gay"`
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)?;

        if !self.1.is_empty() {
            for (name, val) in &self.1 {
                write!(f, " {name}={val:?}")?;
            }
        }

        if !self.2.is_empty() {
            write!(f, " {}", self.2.join(" "))?;
        }

        Ok(())
    }
}

#[derive(Clone)]
pub enum PList {
    Value(String),
    Normal(Tag, Vec<Self>),
    SelfClose(Tag),
    Angry(Tag),
    VeryConfused(Tag),
    Comment(String),
}

impl PList {
    /// plain text, everything else opens/closes a line
    fn is_element(&self) -> bool {
        !matches!(self, Self::Value(_))
    }

    /// the tag of an element node, `None` for text/comments
    pub fn tag(&self) -> Option<&Tag> {
        match self {
            Self::Normal(t, _) | Self::SelfClose(t) | Self::Angry(t) | Self::VeryConfused(t) => {
                Some(t)
            }
            Self::Value(_) | Self::Comment(_) => None,
        }
    }

    /// children of a `<node>`, `None` if it can't have any
    pub fn children(&self) -> Option<&[PList]> {
        match self {
            Self::Normal(_, kids) => Some(kids),
            _ => None,
        }
    }

    /// grow a built tree — returns `None` for everything but [`Self::Normal`]
    pub fn children_mut(&mut self) -> Option<&mut Vec<PList>> {
        match self {
            Self::Normal(_, kids) => Some(kids),
            _ => None,
        }
    }

    fn fmt_at(&self, f: &mut fmt::Formatter<'_>, depth: usize) -> fmt::Result {
        match self {
            Self::Value(v) => f.write_str(v),
            // no element children (text only) -> stay on one line
            Self::Normal(n, v) if !v.iter().any(Self::is_element) => {
                write!(f, "<{n}>")?;
                for kid in v {
                    kid.fmt_at(f, depth)?;
                }
                write!(f, "</{}>", n.name())
            }
            // element children -> one child per line, indented
            Self::Normal(n, v) => {
                writeln!(f, "<{n}>")?;
                for kid in v {
                    indent(f, depth + 1)?;
                    kid.fmt_at(f, depth + 1)?;
                    writeln!(f)?;
                }
                indent(f, depth)?;
                write!(f, "</{}>", n.name())
            }
            Self::SelfClose(n) => write!(f, "<{n}/>"),
            Self::Angry(n) => write!(f, "<!{n}>"),
            Self::VeryConfused(n) => write!(f, "<?{n}?>"),
            Self::Comment(v) => write!(f, "<!-- {v} -->"),
        }
    }
}

impl fmt::Display for PList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_at(f, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_children_stay_inline() {
        assert_eq!(
            PList::Normal(
                Tag::new("string".into(), vec![], vec![]),
                vec![PList::Value("hi".into())]
            )
            .to_string(),
            "<string>hi</string>"
        );
    }

    #[test]
    fn element_children_break_and_indent() {
        let doc = PList::Normal(
            Tag::new(
                "plist".into(),
                vec![("version".into(), "1.0".into())],
                vec![],
            ),
            vec![PList::Normal(
                Tag::new("dict".into(), vec![], vec![]),
                vec![
                    PList::Normal(
                        Tag::new("key".into(), vec![], vec![]),
                        vec![PList::Value("Name".into())],
                    ),
                    PList::Normal(
                        Tag::new("string".into(), vec![], vec![]),
                        vec![PList::Value("Foo".into())],
                    ),
                    PList::SelfClose(Tag::new("true".into(), vec![], vec![])),
                ],
            )],
        );
        assert_eq!(
            doc.to_string(),
            "<plist version=\"1.0\">\n  <dict>\n    <key>Name</key>\n    <string>Foo</string>\n    <true/>\n  </dict>\n</plist>"
        );
    }

    #[test]
    fn children_mut_grows_tree() {
        let mut root = PList::Normal(Tag::new("plist".into(), vec![], vec![]), vec![]);
        root.children_mut().unwrap().push(PList::SelfClose(Tag::new(
            "true".into(),
            vec![],
            vec![],
        )));
        assert_eq!(root.to_string(), "<plist>\n  <true/>\n</plist>");

        assert!(PList::Value("x".into()).children_mut().is_none());
        assert!(PList::Value("x".into()).children().is_none());
        assert!(PList::Value("x".into()).tag().is_none());
        assert_eq!(
            PList::SelfClose(Tag::new("true".into(), vec![], vec![]))
                .tag()
                .unwrap()
                .name(),
            "true"
        );
    }

    #[test]
    fn content_targets_plist_node() {
        let doc = PListFile::new("1.0").content(PList::SelfClose(Tag::new(
            "true".into(),
            vec![],
            vec![],
        )));
        assert!(
            doc.to_string()
                .ends_with("<plist version=\"1.0\">\n  <true/>\n</plist>")
        );
    }

    #[test]
    fn write_serializes_to_file() {
        let path = std::env::temp_dir().join("pissing-pl-test.plist");
        PListFile::new("1.0").write(&path).unwrap();
        let back = std::fs::read_to_string(&path).unwrap();
        assert!(back.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
        std::fs::remove_file(&path).ok();
    }
}
