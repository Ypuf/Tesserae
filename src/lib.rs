use std::path::Path;

mod config;
mod parse;
mod resolve;
mod tessera;
mod value;

pub use config::Config;
pub use parse::{Entry, Parsed, parse};
pub use resolve::resolve;
pub use tessera::apply;
pub use value::{Part, Value};

pub fn load(path: &Path, env: Option<&str>) -> Result<Config, String> {
    let parsed = parse(path)?;
    let base = resolve(apply(parsed.clone(), None)?)?;
    let full = resolve(apply(parsed, env)?)?;

    for (key, (value, line)) in &full {
        let (base_value, base_line) = &base[key];
        if std::mem::discriminant(base_value) != std::mem::discriminant(value) {
            return Err(format!(
                "line {}: '{}' resolved into a different type than its base '{}'",
                line, key, base_line
            ));
        }
    }
    Ok(Config::new(full))
}
