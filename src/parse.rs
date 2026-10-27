use std::fs;
use std::path::Path;

use crate::value::parse_value;

#[derive(Debug)]
pub struct Entry {
    pub tess: Option<String>,
    pub key: String,
    pub value: crate::value::Value,
    pub line: usize,
}

pub fn parse(path: &Path) -> Result<Vec<Entry>, String> {
    let contents =
        fs::read_to_string(path).map_err(|e| format!("couldn't read {}: {}", path.display(), e))?;

    let mut tess: Option<String> = None;
    let mut scope: Vec<String> = Vec::new();
    let mut entries = Vec::new();

    for (index, ln) in contents.lines().enumerate() {
        let line = index + 1;
        let ln = ln.trim();

        if ln.is_empty() || ln.starts_with("//") {
            continue;
        }

        if let Some(rest) = ln.strip_prefix('@') {
            let name = rest.trim();
            tess = if name.is_empty() {
                None
            } else {
                Some(name.to_string())
            };
            scope.clear();
        } else if let Some(rest) = ln.strip_prefix("::") {
            let rest = rest.trim();
            scope = if rest.is_empty() {
                Vec::new()
            } else {
                rest.split('/').map(String::from).collect()
            };
        } else if let Some((key, raw)) = ln.split_once('=') {
            if key.trim().is_empty() {
                return Err(format!("line {}: key can't be empty: '{}'", line, ln));
            }
            let mut parts = scope.clone();
            parts.extend(key.trim().split('/').map(String::from));
            entries.push(Entry {
                tess: tess.clone(),
                key: parts.join("/"),
                value: parse_value(raw.trim(), line)?,
                line,
            });
        } else {
            return Err(format!("line {}: can't parse {:?}", line, ln));
        }
    }

    Ok(entries)
}
