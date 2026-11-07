use std::collections::HashMap;
use std::path::Path;

mod config;
mod error;
mod parse;
mod resolve;
mod tessera;
mod value;

use config::check;
pub use config::{Config, Resolved};
pub use error::{Error, ErrorKind};
use parse::{Parsed, parse, parse_str};
use resolve::resolve;
use tessera::apply;
pub use value::ValueKind;
use value::{Part, Value};

type Map = HashMap<String, (Value, usize)>;

pub fn load(path: &Path, env: Option<&str>) -> Result<Config, Error> {
    parse(path)
        .and_then(|parsed| build(parsed, env, Some(path)))
        .map_err(|e| e.in_file(Some(path)))
}

pub fn load_str(src: &str, env: Option<&str>) -> Result<Config, Error> {
    build(parse_str(src)?, env, None)
}

fn build(parsed: Parsed, env: Option<&str>, path: Option<&Path>) -> Result<Config, Error> {
    let base = resolve(apply(parsed.clone(), None)?)?;
    check(&base)?;

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

    Ok(Config::new(requested, path.map(Path::to_path_buf)))
}

fn overlay(parsed: &Parsed, name: &str, base: &Map) -> Result<Map, Error> {
    let full = resolve(apply(parsed.clone(), Some(name))?).map_err(|e| e.in_tessera(name))?;

    for (key, (value, line)) in &full {
        let (base_value, base_line) = &base[key];
        if !base_value.same_type(value) {
            return Err(Error::at(
                *line,
                ErrorKind::ResolvedTypeMismatch {
                    tessera: name.to_string(),
                    key: key.clone(),
                    base_line: *base_line,
                },
            ));
        }
    }

    check(&full).map_err(|e| e.in_tessera(name))?;

    Ok(full)
}
