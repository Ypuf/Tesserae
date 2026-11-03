use std::env;
use std::path::Path;
use tesserae::load;

fn main() {
    let mut args = env::args();
    let path_arg = args.nth(1).expect("usage: <path> [env]");
    let env_name: Option<String> = args.next();

    let result = load(Path::new(&path_arg), env_name.as_deref());

    match result {
        Ok(config) => {
            let map = config.raw();
            let mut keys: Vec<_> = map.keys().collect();
            keys.sort();
            for k in keys {
                let (value, line) = &map[k];
                println!("{} = {:?}  (line {})", k, value, line);
            }
        }
        Err(msg) => {
            eprintln!("{}", msg);
            std::process::exit(1);
        }
    }
}
