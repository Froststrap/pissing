pub mod pl;

use pl::*;

pub fn main() {
    let doc = PListFile::new(1.0);

    doc.display()
}
