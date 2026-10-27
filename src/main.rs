use std::env;
use std::path::Path;
use tesserae::{apply, parse};

fn main() {
    let env_name = env::args().nth(1);

    let result = parse(Path::new("file/example.tess"))
        .and_then(|entries| apply(entries, env_name.as_deref()));

    match result {
        Ok(map) => {
            let mut keys: Vec<_> = map.keys().collect();
            keys.sort();
            for k in keys {
                let (value, line) = &map[k];
                println!("{} = {:?}  (line {})", k, value, line);
            }
        }
        Err(msg) => eprintln!("{}", msg),
    }
}
