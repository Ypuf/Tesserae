use std::env;
use std::path::Path;
use tesserae::{Config, load};

fn render(config: &Config) -> Result<String, String> {
    config
        .keys()
        .into_iter()
        .map(|k| {
            let (value, line) = (config.get(k)?, config.line(k)?);
            Ok(format!("{} = {:?}  (line {})\n", k, value, line))
        })
        .collect()
}

fn main() {
    let mut args = env::args();
    let path_arg = args.nth(1).expect("usage: <path> [env]");
    let env_name: Option<String> = args.next();

    let result = load(Path::new(&path_arg), env_name.as_deref()).and_then(|c| render(&c));

    match result {
        Ok(output) => print!("{}", output),
        Err(msg) => {
            eprintln!("{}", msg);
            std::process::exit(1);
        }
    }
}
