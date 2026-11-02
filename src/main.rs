use std::env;
use std::path::Path;
use tesserae::load;

fn main() {
    let mut args = env::args();
    let path_arg = args.nth(1).expect("usage: <path> [env]");
    let fuckyou: Option<String> = args.next();
    let env_name: Option<&str> = fuckyou.as_deref();

    let result = load(Path::new(&path_arg), env_name);

    match result {
        Ok(map) => {
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
