use pissing::prelude::*;

fn main() {
    let doc = PListFile::new("1.0").content(dict![
        string("Test", "Gooooodwondqowinw"),
        bool("dog_is_dead", true),
        dict("racism", [string("buh", "asteyo")])
    ]);

    doc.print();

    if let Err(e) = doc.write("target/Info.plist") {
        eprintln!("could not write target/Info.plist: {e}");
    }
}
