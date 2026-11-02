use std::env;
use std::path::Path;
use tesserae::{apply, parse, resolve};

fn main() {
    let env_name = env::args().nth(1);

    let result = parse(Path::new("file/example.tess"))
        .and_then(|parsed| apply(parsed, env_name.as_deref()))
        .and_then(resolve);

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
