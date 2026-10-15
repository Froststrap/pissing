//! typed construction on top of the core.

use crate::pl::{PList, Tag};

pub struct Entry {
    key: String,
    value: PList,
}

impl Entry {
    pub fn new(key: impl Into<String>, value: PList) -> Self {
        Self {
            key: key.into(),
            value,
        }
    }

    pub fn into_nodes(self) -> [PList; 2] {
        [
            PList::Normal(
                Tag::new("key".into(), vec![], vec![]),
                vec![PList::Value(self.key)],
            ),
            self.value,
        ]
    }
}

/// an element with children, e.g `node("dict", kids)`
pub fn node(name: &str, children: Vec<PList>) -> PList {
    PList::Normal(Tag::new(name.to_string(), vec![], vec![]), children)
}

fn leaf(name: &str, text: String) -> PList {
    node(name, vec![PList::Value(text)])
}

/// converts a literal (`&str`, `bool`, int, float) into its node
pub fn to_node<T: Into<PList>>(value: T) -> PList {
    value.into()
}

/// generic key + value entry for when the typed fns don't fit
pub fn entry(key: impl Into<String>, value: impl Into<PList>) -> Entry {
    Entry::new(key, value.into())
}

pub fn string(key: impl Into<String>, value: impl Into<String>) -> Entry {
    Entry::new(key, leaf("string", value.into()))
}

pub fn bool(key: impl Into<String>, value: bool) -> Entry {
    Entry::new(key, value.into())
}

pub fn integer(key: impl Into<String>, value: i64) -> Entry {
    Entry::new(key, leaf("integer", value.to_string()))
}

pub fn real(key: impl Into<String>, value: f64) -> Entry {
    Entry::new(key, leaf("real", format!("{value:?}")))
}

pub fn date(key: impl Into<String>, value: impl Into<String>) -> Entry {
    Entry::new(key, leaf("date", value.into()))
}

pub fn data(key: impl Into<String>, bytes: impl AsRef<[u8]>) -> Entry {
    Entry::new(key, leaf("data", base64(bytes.as_ref())))
}

pub fn array<I>(key: impl Into<String>, items: I) -> Entry
where
    I: IntoIterator,
    I::Item: Into<PList>,
{
    let kids: Vec<PList> = items.into_iter().map(Into::into).collect();
    Entry::new(key, node("array", kids))
}

pub fn dict<I: IntoIterator<Item = Entry>>(key: impl Into<String>, entries: I) -> Entry {
    Entry::new(key, dict_node(entries))
}

pub fn dict_node<I: IntoIterator<Item = Entry>>(entries: I) -> PList {
    let mut kids = Vec::new();
    for e in entries {
        kids.extend(e.into_nodes());
    }
    node("dict", kids)
}

#[macro_export]
macro_rules! dict {
    ($($entry:expr),* $(,)?) => {{
        #[allow(unused_mut)]
        let mut kids = ::std::vec::Vec::new();
        $(kids.extend($crate::builder::Entry::into_nodes($entry));)*
        $crate::builder::node("dict", kids)
    }};
}

#[macro_export]
macro_rules! array {
    ($($item:expr),* $(,)?) => {{
        #[allow(unused_mut)]
        let mut kids = ::std::vec::Vec::new();
        $(kids.push($crate::builder::to_node($item));)*
        $crate::builder::node("array", kids)
    }};
}

fn base64(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b = [
            chunk[0],
            *chunk.get(1).unwrap_or(&0),
            *chunk.get(2).unwrap_or(&0),
        ];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        out.push(TABLE[((n >> 18) as usize) & 63] as char);
        out.push(TABLE[((n >> 12) as usize) & 63] as char);
        out.push(if chunk.len() > 1 {
            TABLE[((n >> 6) as usize) & 63] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            TABLE[(n as usize) & 63] as char
        } else {
            '='
        });
    }
    out
}

impl From<&str> for PList {
    fn from(value: &str) -> Self {
        leaf("string", value.to_string())
    }
}

impl From<String> for PList {
    fn from(value: String) -> Self {
        leaf("string", value)
    }
}

impl From<bool> for PList {
    fn from(value: bool) -> Self {
        PList::SelfClose(Tag::new(
            if value { "true" } else { "false" }.to_string(),
            vec![],
            vec![],
        ))
    }
}

impl From<f64> for PList {
    fn from(value: f64) -> Self {
        leaf("real", format!("{value:?}"))
    }
}

impl From<f32> for PList {
    fn from(value: f32) -> Self {
        leaf("real", format!("{value:?}"))
    }
}

macro_rules! from_ints {
    ($($t:ty),+ $(,)?) => {
        $(impl From<$t> for PList {
            fn from(value: $t) -> Self {
                leaf("integer", value.to_string())
            }
        })+
    };
}

from_ints!(i8, i16, i32, i64, isize, u8, u16, u32, u64, usize);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pl::PListFile;

    #[test]
    fn literal_conversions() {
        assert_eq!(to_node("hi").to_string(), "<string>hi</string>");
        assert_eq!(to_node(true).to_string(), "<true/>");
        assert_eq!(to_node(false).to_string(), "<false/>");
        assert_eq!(to_node(42).to_string(), "<integer>42</integer>");
        assert_eq!(to_node(1.5).to_string(), "<real>1.5</real>");
        assert_eq!(to_node(1.0).to_string(), "<real>1.0</real>");
    }

    #[test]
    fn entries_flatten_into_pairs() {
        assert_eq!(
            string("Name", "App").into_nodes()[0].to_string(),
            "<key>Name</key>"
        );
        assert_eq!(
            string("Name", "App").into_nodes()[1].to_string(),
            "<string>App</string>"
        );
        assert_eq!(bool("On", true).into_nodes()[1].to_string(), "<true/>");
        assert_eq!(bool("On", false).into_nodes()[1].to_string(), "<false/>");
    }

    #[test]
    fn data_is_base64() {
        assert_eq!(
            data("Icon", b"foobar").into_nodes()[1].to_string(),
            "<data>Zm9vYmFy</data>"
        );
    }

    #[test]
    fn array_entry_converts_items() {
        assert_eq!(
            array("List", ["a", "b"]).into_nodes()[1].to_string(),
            "<array>\n  <string>a</string>\n  <string>b</string>\n</array>"
        );
    }

    #[test]
    fn dict_macro_nests() {
        assert_eq!(
            dict![dict("Outer", [string("a", "b")])].to_string(),
            "<dict>\n  <key>Outer</key>\n  <dict>\n    <key>a</key>\n    <string>b</string>\n  </dict>\n</dict>"
        );
    }

    #[test]
    fn array_macro_converts_each_literal() {
        assert_eq!(
            array!["a", true, 3].to_string(),
            "<array>\n  <string>a</string>\n  <true/>\n  <integer>3</integer>\n</array>"
        );
    }

    #[test]
    fn full_document() {
        let doc = PListFile::new("1.0").content(dict![
            string("CFBundleName", "MyApp"),
            bool("LSRequiresIPhoneOS", true),
            integer("CFBundleVersion", 3),
        ]);
        assert!(doc.to_string().ends_with(
            "<plist version=\"1.0\">\n  <dict>\n    <key>CFBundleName</key>\n    <string>MyApp</string>\n    <key>LSRequiresIPhoneOS</key>\n    <true/>\n    <key>CFBundleVersion</key>\n    <integer>3</integer>\n  </dict>\n</plist>"
        ));
    }

    #[test]
    fn write_round_trips_to_file() {
        let path = std::env::temp_dir().join("pissing-builder-test.plist");
        let doc = PListFile::new("1.0").content(dict![string("k", "v")]);
        doc.write(&path).unwrap();
        let back = std::fs::read_to_string(&path).unwrap();
        assert!(back.contains("<key>k</key>"));
        std::fs::remove_file(&path).ok();
    }
}
