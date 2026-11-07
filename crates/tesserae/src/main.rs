use std::env;
use std::error::Error as _;
use std::path::Path;
use std::process;
use tesserae::{Config, Error, load};

fn render(config: &Config) -> Result<String, Error> {
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
    let Some(path_arg) = args.nth(1) else {
        eprintln!("usage: <path> [env]");
        process::exit(2);
    };
    let env_name: Option<String> = args.next();

    let result = load(Path::new(&path_arg), env_name.as_deref()).and_then(|c| render(&c));

    match result {
        Ok(output) => print!("{}", output),
        Err(e) => {
            eprint!("{}", e);
            let mut source = e.source();
            while let Some(cause) = source {
                eprint!(": {}", cause);
                source = cause.source();
            }
            eprintln!();
            process::exit(1);
        }
    }
}
