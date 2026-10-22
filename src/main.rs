// use regex::Regex;
use std::fs;
use std::path::Path;

#[allow(dead_code)] // temp, its just annoying me
#[derive(Debug)]
struct Entry {
    tess: Option<String>,
    key: String,
    raw: String,
    line: usize,
}

fn parse(path: &Path) -> Result<Vec<Entry>, String> {
    let contents =
        fs::read_to_string(path).map_err(|e| format!("couldn't read {}: {}", path.display(), e))?;

    let mut env: Option<String> = None;
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
            env = if name.is_empty() {
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
            let mut parts = scope.clone();
            parts.extend(key.trim().split('/').map(String::from));
            entries.push(Entry {
                tess: env.clone(),
                key: parts.join("/"),
                raw: raw.trim().to_string(),
                line,
            });
        } else {
            return Err(format!("line {}: can't parse {:?}", line, ln));
        }
    }

    Ok(entries)
}

fn main() {
    match parse(Path::new("file/example.tess")) {
        Ok(entries) => {
            for e in &entries {
                println!("{:?}", e);
            }
        }
        Err(msg) => eprintln!("{}", msg),
    }
}
