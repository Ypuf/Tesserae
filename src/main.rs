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

#[allow(dead_code)] // temp, until overlays/refs read these
#[derive(Debug)]
enum Value {
    Int(i64),
    Float(f64),
    Str(Vec<Part>),
    Port(u16),
    Duration(f64, String),
    Size(f64, String),
    Colour(String),
    Ref(String),
}

#[allow(dead_code)]
#[derive(Debug)]
enum Part {
    Text(String),
    Ref(String),
}

const DUR: &[&str] = &["sec", "min", "hr", "wk", "mo", "yr", "d"];
const SIZE: &[&str] = &["KiB", "MiB", "GiB", "TiB", "kB", "MB", "GB", "TB", "b", "B"];

fn parse_value(raw: &str, line: usize) -> Result<Value, String> {
    if let Some(num) = raw.strip_prefix(':') {
        return match num.parse::<u16>() {
            Ok(port) => Ok(Value::Port(port)),
            Err(_) => Err(format!("line {}: invalid port: '{}'", line, num)),
        };
    }

    if let Some(hexpart) = raw.strip_prefix("#") {
        if !matches!(hexpart.len(), 6 | 8) || !hexpart.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(format!("line {}: invalid colour: '{}'", line, raw));
        }
        return Ok(Value::Colour(hexpart.to_ascii_uppercase()));
    }

    if let Some(rest) = raw.strip_prefix('"') {
        return match rest.strip_suffix('"') {
            Some(inner) => parse_string(inner, line),
            None => Err(format!("line {}: unterminated string: {}", line, raw)),
        };
    }

    if let Ok(n) = raw.parse::<i64>() {
        return Ok(Value::Int(n));
    }

    if raw.contains('.') {
        if let Ok(f) = raw.parse::<f64>() {
            return Ok(Value::Float(f));
        }
    }

    if let Some((n, unit)) = split_unit(raw, DUR) {
        return Ok(Value::Duration(n, unit));
    }

    if let Some((n, unit)) = split_unit(raw, SIZE) {
        return Ok(Value::Size(n, unit));
    }

    Err(format!("line {}: unknown value: '{}'", line, raw))
}

fn split_unit(raw: &str, units: &[&str]) -> Option<(f64, String)> {
    units.iter().find_map(|unit| {
        let num = raw.strip_suffix(unit)?;
        let digits_ok = num.starts_with(|c: char| c.is_ascii_digit())
            && num.chars().all(|c| c.is_ascii_digit() || c == '.')
            && num.matches('.').count() <= 1;
        if !digits_ok {
            return None;
        }
        let n = num.parse::<f64>().ok()?;
        Some((n, unit.to_string()))
    })
}

fn parse_string(inner: &str, line: usize) -> Result<Value, String> {
    let mut parts = Vec::new();
    let mut buf = String::new();
    let mut chars = inner.chars();

    while let Some(c) = chars.next() {
        match c {
            '\\' => match chars.next() {
                Some(e @ ('"' | '\\' | '{' | '}')) => buf.push(e),
                _ => return Err(format!("line {}: bad escape in \"{}\"", line, inner)),
            },
            '{' => {
                let mut path = String::new();
                loop {
                    match chars.next() {
                        Some('}') => break,
                        Some(c) => path.push(c),
                        None => return Err(format!("line {}: unclosed {{ in \"{}\"", line, inner)),
                    }
                }
                if path.is_empty() {
                    return Err(format!("line {}: empty reference in \"{}\"", line, inner));
                }
                if !buf.is_empty() {
                    parts.push(Part::Text(std::mem::take(&mut buf)));
                }
                parts.push(Part::Ref(path));
            }
            '}' => return Err(format!("line {}: unmatched }} in \"{}\"", line, inner)),
            '"' => return Err(format!("line {}: unescaped quote in \"{}\"", line, inner)),
            _ => buf.push(c),
        }
    }

    if !buf.is_empty() {
        parts.push(Part::Text(buf));
    }

    if let [Part::Ref(path)] = parts.as_slice() {
        return Ok(Value::Ref(path.clone()));
    }

    Ok(Value::Str(parts))
}

fn parse(path: &Path) -> Result<Vec<Entry>, String> {
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
                return Err(format!("line {}: key can't be empty: '{}'", line, ln))
            }
            let mut parts = scope.clone();
            parts.extend(key.trim().split('/').map(String::from));
            entries.push(Entry {
                tess: tess.clone(),
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
                match parse_value(&e.raw, e.line) {
                    Ok(v) => println!("{:?} {} = {:?}", e.tess, e.key, v),
                    Err(msg) => eprintln!("{}", msg),
                }
            }
        }
        Err(msg) => eprintln!("{}", msg),
    }
}
