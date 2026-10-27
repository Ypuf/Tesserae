use std::path::Path;
use tesserae::parse;

fn main() {
    match parse(Path::new("file/example.tess")) {
        Ok(entries) => {
            for e in &entries {
                println!("{:?} {} = {:?}", e.tess, e.key, e.value);
            }
        }
        Err(msg) => eprintln!("{}", msg),
    }
}
