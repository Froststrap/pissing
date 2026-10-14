//! support clean data structures for plist.

/// simplify the creation of a plist file with `PListTopLevel`
pub struct PListFile(PListTopLevel);

impl PListFile {
    /// creates a `PListTopLevel` with starting template.
    /// Use `PlistTopLevel` if you don't want abstraction- rather building top-level yourself.
    pub fn new(pl_ver: f32) -> Self {
        let doc = PListTopLevel::new()
            .head(PList::VeryConfused(Tag::new(
                "xml".into(),
                vec![
                    ("version".into(), "1.0".into()),
                    ("encoding".into(), "UTF-8".into())
                ],
                vec![]
            )))
            .head(PList::Angry(Tag::new(
                "DOCTYPE".into(),
                vec![],
                vec![
                    "plist".into(),
                    "PUBLIC".into(),
                    "\"-//Apple//DTD PLIST 1.0//EN\"".into(),
                    "\"http://www.apple.com/DTDs/PropertyList-1.0.dtd\"".into(),
                ]
            )))
            .body(
                PList::Normal(
                    Tag::new(
                        "plist".into(),
                        vec![
                            ("version".into(), pl_ver.to_string())
                        ],
                         vec![]
                     ),
                     vec![]
                )
            );

        PListFile(doc)
    }

    pub fn write(self) -> String {
        self.0.write()
    }
    pub fn display(self) {
        println!("{}", self.write());
    }
}

pub struct PListTopLevel(Vec<PList>);

impl PListTopLevel {
    pub fn new() -> Self {
        PListTopLevel(vec![])
    }

    pub fn head(mut self, item: PList) -> Self {
        match item {
            PList::Value(_)
            | PList::Normal(_, _) => panic!("unable to express in top level of XML"),
            _ => {
                self.0.push(item);
                self
            }, 
        } 
    }

    pub fn body(mut self, item:PList) -> Self {
        self.0.push(item);
        self
    }
}

impl PListTopLevel {
    pub fn remove(mut self, idx: usize) -> Self {
        self.0.remove(idx);
        self
    }
    pub fn write(self) -> String {
        let l: Vec<String> = self.0.iter().map(|node| node.clone().write()).collect();

        l.join("\n")
    }
    pub fn display(self) {
        println!("{}", self.write());
    }
}

/// Means can be `<name x="2430">` or `<name class>` or `<name>`.
#[derive(Clone)]
pub struct Tag(String, Vec<(String, String)>, Vec<String>);

impl Tag {
    pub fn new(name: String, assigns: Vec<(String, String)>, classes: Vec<String>) -> Self {
        Self(name, assigns, classes)
    }

    /// simple wrapper to just make as a string the whole blob to be evaluated
    /// e.g `Tag("e", vec![("w", "s")], vec!["gay"])` -> `"e w=s gay"`
    pub fn write(self) -> String {
        format!(
            "{}{}{}{}{}",
            self.0,
            if self.1.is_empty() { "" } else { " " },
            self.1.iter().map(|(name, val)| format!("{name}={val:?}")).collect::<Vec<String>>().join(" "),
            if self.2.is_empty() { "" } else { " " },
            self.2.join(" ")
        )
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
    pub fn write(self) -> String {
        match self {
            Self::Value(v) => v,
            Self::Normal(n, v) => format!("<{}>{}</{}>", n.clone().write(), v.iter().map(|n| n.clone().write()).collect::<Vec<String>>().join("\n"), n.0),
            Self::SelfClose(n) => format!("<{}/>", n.write()),
            Self::Angry(n) => format!("<!{}>", n.write()),
            Self::VeryConfused(n) => format!("<?{}?>", n.write()),
            Self::Comment(v) => format!("<!-- {v} -->"),
        }
    }
}
