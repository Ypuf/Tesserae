use std::collections::HashSet;
use std::fs;
use std::path::Path;

use crate::value::parse_value;

#[derive(Debug, Clone)]
pub(crate) struct Entry {
    pub(crate) tess: Option<String>,
    pub(crate) key: String,
    pub(crate) value: crate::value::Value,
    pub(crate) line: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct Parsed {
    pub(crate) entries: Vec<Entry>,
    pub(crate) tesserae: HashSet<String>,
}

fn strip_comment(ln: &str) -> &str {
    let mut in_string = false;
    let mut escaped = false;

    for (i, c) in ln.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                in_string = false;
            }
        } else if c == '"' {
            in_string = true;
        } else if ln[i..].starts_with("//") && (i == 0 || ln[..i].ends_with(char::is_whitespace)) {
            return &ln[..i];
        }
    }
    ln
}

fn valid_name(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

fn valid_path(s: &str) -> bool {
    s.split('/').all(valid_name)
}

pub(crate) fn parse(path: &Path) -> Result<Parsed, String> {
    let contents =
        fs::read_to_string(path).map_err(|e| format!("couldn't read {}: {}", path.display(), e))?;

    parse_str(&contents)
}

pub(crate) fn parse_str(contents: &str) -> Result<Parsed, String> {
    let mut tess: Option<String> = None;
    let mut scope: Vec<String> = Vec::new();
    let mut entries = Vec::new();
    let mut tesserae: HashSet<String> = HashSet::new();

    for (index, ln) in contents.lines().enumerate() {
        let line = index + 1;
        let ln = strip_comment(ln).trim();

        if ln.is_empty() {
            continue;
        }

        if let Some(rest) = ln.strip_prefix('@') {
            let name = rest.trim();
            tess = if name.is_empty() {
                None
            } else {
                if !valid_name(name) {
                    return Err(format!("line {}: tessera name is invalid: '{}'", line, ln));
                }
                tesserae.insert(name.to_string());
                Some(name.to_string())
            };
            scope.clear();
        } else if let Some(rest) = ln.strip_prefix("::") {
            let rest = rest.trim();
            scope = if rest.is_empty() {
                Vec::new()
            } else {
                if !valid_path(rest) {
                    return Err(format!("line {}: path name is invalid: '{}'", line, ln));
                }
                rest.split('/').map(String::from).collect()
            };
        } else if let Some((key, raw)) = ln.split_once('=') {
            if key.trim().is_empty() {
                return Err(format!("line {}: key can't be empty: '{}'", line, ln));
            }
            if !valid_path(key.trim()) {
                return Err(format!("line {}: key name is invalid: '{}'", line, ln));
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

    Ok(Parsed { entries, tesserae })
}
