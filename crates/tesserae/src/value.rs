use std::fmt;
use std::mem::discriminant;

use crate::error::{Error, ErrorKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ValueKind {
    Bool,
    Int,
    Float,
    Str,
    Port,
    Duration,
    Size,
    Colour,
    List,
}

impl fmt::Display for ValueKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            ValueKind::Bool => "a bool",
            ValueKind::Int => "an int",
            ValueKind::Float => "a float",
            ValueKind::Str => "a string",
            ValueKind::Port => "a port",
            ValueKind::Duration => "a duration",
            ValueKind::Size => "a size",
            ValueKind::Colour => "a colour",
            ValueKind::List => "a list",
        })
    }
}

#[derive(Debug, Clone)]
pub(crate) enum Value {
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(Vec<Part>),
    Port(u16),
    Duration(f64, String),
    Size(f64, String),
    Colour(String),
    Ref(String),
    List(Vec<Value>),
}

impl Value {
    pub(crate) fn kind(&self) -> ValueKind {
        match self {
            Value::Bool(_) => ValueKind::Bool,
            Value::Int(_) => ValueKind::Int,
            Value::Float(_) => ValueKind::Float,
            Value::Str(_) | Value::Ref(_) => ValueKind::Str,
            Value::Port(_) => ValueKind::Port,
            Value::Duration(..) => ValueKind::Duration,
            Value::Size(..) => ValueKind::Size,
            Value::Colour(_) => ValueKind::Colour,
            Value::List(_) => ValueKind::List,
        }
    }

    pub(crate) fn same_type(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::List(a), Value::List(b)) => match (a.first(), b.first()) {
                (Some(x), Some(y)) => discriminant(x) == discriminant(y),
                _ => true,
            },
            _ => discriminant(self) == discriminant(other),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum Part {
    Text(String),
    Ref(String),
}

const DUR: &[&str] = &["sec", "min", "hr", "wk", "mo", "yr", "d"];
const SIZE: &[&str] = &[
    "KiB", "MiB", "GiB", "TiB", "kB", "KB", "MB", "GB", "TB", "B",
];

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

fn parse_string(inner: &str, line: usize) -> Result<Value, Error> {
    let string = || inner.to_string();
    let mut parts = Vec::new();
    let mut buf = String::new();
    let mut chars = inner.chars();

    while let Some(c) = chars.next() {
        match c {
            '\\' => match chars.next() {
                Some(e @ ('"' | '\\' | '{' | '}')) => buf.push(e),
                _ => {
                    return Err(Error::at(line, ErrorKind::BadEscape { string: string() }));
                }
            },
            '{' => {
                let mut path = String::new();
                loop {
                    match chars.next() {
                        Some('}') => break,
                        Some(c) => path.push(c),
                        None => {
                            return Err(Error::at(
                                line,
                                ErrorKind::UnclosedReference { string: string() },
                            ));
                        }
                    }
                }
                if path.is_empty() {
                    return Err(Error::at(
                        line,
                        ErrorKind::EmptyReference { string: string() },
                    ));
                }
                if !buf.is_empty() {
                    parts.push(Part::Text(std::mem::take(&mut buf)));
                }
                parts.push(Part::Ref(path));
            }
            '}' => {
                return Err(Error::at(
                    line,
                    ErrorKind::UnmatchedBrace { string: string() },
                ));
            }
            '"' => {
                return Err(Error::at(
                    line,
                    ErrorKind::UnescapedQuote { string: string() },
                ));
            }
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

fn split_items(inner: &str) -> Vec<&str> {
    let mut items = Vec::new();
    let mut start = 0;
    let mut in_string = false;
    let mut escaped = false;

    for (i, c) in inner.char_indices() {
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
        } else if c == ',' {
            items.push(inner[start..i].trim());
            start = i + 1;
        }
    }

    items.push(inner[start..].trim());
    items
}

fn parse_list(raw: &str, line: usize) -> Result<Value, Error> {
    let Some(inner) = raw.strip_prefix('[').and_then(|r| r.strip_suffix(']')) else {
        return Err(Error::at(
            line,
            ErrorKind::UnterminatedList {
                list: raw.to_string(),
            },
        ));
    };

    if inner.trim().is_empty() {
        return Ok(Value::List(Vec::new()));
    }

    let mut items = Vec::new();
    for item in split_items(inner) {
        if item.is_empty() {
            return Err(Error::at(
                line,
                ErrorKind::EmptyListItem {
                    list: raw.to_string(),
                },
            ));
        }
        if item.starts_with('[') {
            return Err(Error::at(
                line,
                ErrorKind::NestedList {
                    list: Some(raw.to_string()),
                },
            ));
        }
        items.push(parse_value(item, line)?);
    }

    Ok(Value::List(items))
}

pub(crate) fn parse_value(raw: &str, line: usize) -> Result<Value, Error> {
    if raw.starts_with('[') {
        return parse_list(raw, line);
    }

    if let Some(num) = raw.strip_prefix(':') {
        return match num.parse::<u16>() {
            Ok(port) if !num.starts_with('+') => Ok(Value::Port(port)),
            _ => Err(Error::at(
                line,
                ErrorKind::InvalidPort {
                    value: num.to_string(),
                },
            )),
        };
    }

    if let Some(hexpart) = raw.strip_prefix("#") {
        if !matches!(hexpart.len(), 6 | 8) || !hexpart.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(Error::at(
                line,
                ErrorKind::InvalidColour {
                    value: raw.to_string(),
                },
            ));
        }
        return Ok(Value::Colour(hexpart.to_ascii_uppercase()));
    }

    if let Some(rest) = raw.strip_prefix('"') {
        return match rest.strip_suffix('"') {
            Some(inner) => parse_string(inner, line),
            None => Err(Error::at(
                line,
                ErrorKind::UnterminatedString {
                    value: raw.to_string(),
                },
            )),
        };
    }

    match raw {
        "true" => return Ok(Value::Bool(true)),
        "false" => return Ok(Value::Bool(false)),
        _ => {}
    }

    let unknown = || {
        Error::at(
            line,
            ErrorKind::UnknownValue {
                value: raw.to_string(),
            },
        )
    };

    if raw.starts_with('+') {
        return Err(unknown());
    }

    if let Ok(n) = raw.parse::<i64>() {
        return Ok(Value::Int(n));
    }

    if raw.contains('.')
        && let Ok(f) = raw.parse::<f64>()
    {
        if f.is_infinite() {
            return Err(Error::at(
                line,
                ErrorKind::FloatOutOfRange {
                    value: raw.to_string(),
                },
            ));
        }
        return Ok(Value::Float(f));
    }

    if let Some((n, unit)) = split_unit(raw, DUR) {
        return Ok(Value::Duration(n, unit));
    }

    if let Some((n, unit)) = split_unit(raw, SIZE) {
        return Ok(Value::Size(n, unit));
    }

    Err(unknown())
}
