#[derive(Debug, Clone)]
pub enum Value {
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(Vec<Part>),
    Port(u16),
    Duration(f64, String),
    Size(f64, String),
    Colour(String),
    Ref(String),
}

#[derive(Debug, Clone)]
pub enum Part {
    Text(String),
    Ref(String),
}

const DUR: &[&str] = &["sec", "min", "hr", "wk", "mo", "yr", "d"];
const SIZE: &[&str] = &["KiB", "MiB", "GiB", "TiB", "kB", "MB", "GB", "TB", "b", "B"];

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

pub fn parse_value(raw: &str, line: usize) -> Result<Value, String> {
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

    match raw {
        "true" => return Ok(Value::Bool(true)),
        "false" => return Ok(Value::Bool(false)),
        _ => {}
    }

    if let Ok(n) = raw.parse::<i64>() {
        return Ok(Value::Int(n));
    }

    if raw.contains('.')
        && let Ok(f) = raw.parse::<f64>()
    {
        return Ok(Value::Float(f));
    }

    if let Some((n, unit)) = split_unit(raw, DUR) {
        return Ok(Value::Duration(n, unit));
    }

    if let Some((n, unit)) = split_unit(raw, SIZE) {
        return Ok(Value::Size(n, unit));
    }

    Err(format!("line {}: unknown value: '{}'", line, raw))
}
