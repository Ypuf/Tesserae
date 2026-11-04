use std::collections::HashMap;
use std::path::Path;

mod config;
mod parse;
mod resolve;
mod tessera;
mod value;

pub use config::Config;
use parse::parse_str;
pub use parse::{Entry, Parsed, parse};
pub use resolve::resolve;
pub use tessera::apply;
pub use value::{Part, Value};

type Map = HashMap<String, (Value, usize)>;

pub fn load(path: &Path, env: Option<&str>) -> Result<Config, String> {
    build(parse(path)?, env)
}

pub fn load_str(src: &str, env: Option<&str>) -> Result<Config, String> {
    build(parse_str(src)?, env)
}

fn build(parsed: Parsed, env: Option<&str>) -> Result<Config, String> {
    let base = resolve(apply(parsed.clone(), None)?)?;

    let requested = match env {
        Some(name) => overlay(&parsed, name, &base)?,
        None => base.clone(),
    };

    let mut others: Vec<&str> = parsed
        .tesserae
        .iter()
        .map(String::as_str)
        .filter(|name| Some(*name) != env)
        .collect();
    others.sort();

    for name in others {
        overlay(&parsed, name, &base)?;
    }

    Ok(Config::new(requested))
}

fn overlay(parsed: &Parsed, name: &str, base: &Map) -> Result<Map, String> {
    let full = resolve(apply(parsed.clone(), Some(name))?).map_err(|e| format!("@{name}: {e}"))?;

    for (key, (value, line)) in &full {
        let (base_value, base_line) = &base[key];
        if !base_value.same_type(value) {
            return Err(format!(
                "line {}: @{} resolves '{}' to a different type than base (line {})",
                line, name, key, base_line
            ));
        }
    }

    Ok(full)
}
