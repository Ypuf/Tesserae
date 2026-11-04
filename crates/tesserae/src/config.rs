use crate::{Part, Value};
use std::collections::HashMap;
use std::time::Duration;

pub struct Config {
    map: HashMap<String, (Value, usize)>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Resolved {
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
    Port(u16),
    Duration(Duration),
    Size(u64),
    Colour(String),
    List(Vec<Resolved>),
}

type Read<'a, T> = fn(&str, &'a Value, usize) -> Result<T, String>;

impl Config {
    pub(crate) fn new(map: HashMap<String, (Value, usize)>) -> Self {
        Config { map }
    }

    pub fn keys(&self) -> Vec<&str> {
        let mut keys: Vec<&str> = self.map.keys().map(String::as_str).collect();
        keys.sort();
        keys
    }

    pub fn get(&self, key: &str) -> Result<Resolved, String> {
        self.one(key, read_resolved)
    }

    pub fn line(&self, key: &str) -> Result<usize, String> {
        self.entry(key).map(|(_, line)| *line)
    }

    fn entry(&self, key: &str) -> Result<&(Value, usize), String> {
        self.map
            .get(key)
            .ok_or_else(|| format!("no key '{}' in config", key))
    }

    fn one<'a, T>(&'a self, key: &str, read: Read<'a, T>) -> Result<T, String> {
        let (value, line) = self.entry(key)?;
        read(key, value, *line)
    }

    fn many<'a, T>(&'a self, key: &str, read: Read<'a, T>) -> Result<Vec<T>, String> {
        match self.entry(key)? {
            (Value::List(items), line) => items
                .iter()
                .enumerate()
                .map(|(i, item)| read(&format!("{}[{}]", key, i), item, *line))
                .collect(),
            (v, line) => Err(wrong(key, *line, "a list", v)),
        }
    }

    pub fn str(&self, key: &str) -> Result<&str, String> {
        self.one(key, read_str)
    }

    pub fn strs(&self, key: &str) -> Result<Vec<&str>, String> {
        self.many(key, read_str)
    }

    pub fn port(&self, key: &str) -> Result<u16, String> {
        self.one(key, read_port)
    }

    pub fn ports(&self, key: &str) -> Result<Vec<u16>, String> {
        self.many(key, read_port)
    }

    pub fn int(&self, key: &str) -> Result<i64, String> {
        self.one(key, read_int)
    }

    pub fn ints(&self, key: &str) -> Result<Vec<i64>, String> {
        self.many(key, read_int)
    }

    pub fn float(&self, key: &str) -> Result<f64, String> {
        self.one(key, read_float)
    }

    pub fn floats(&self, key: &str) -> Result<Vec<f64>, String> {
        self.many(key, read_float)
    }

    pub fn colour(&self, key: &str) -> Result<String, String> {
        self.one(key, read_colour)
    }

    pub fn colours(&self, key: &str) -> Result<Vec<String>, String> {
        self.many(key, read_colour)
    }

    pub fn bool(&self, key: &str) -> Result<bool, String> {
        self.one(key, read_bool)
    }

    pub fn bools(&self, key: &str) -> Result<Vec<bool>, String> {
        self.many(key, read_bool)
    }

    pub fn duration(&self, key: &str) -> Result<Duration, String> {
        self.one(key, read_duration)
    }

    pub fn durations(&self, key: &str) -> Result<Vec<Duration>, String> {
        self.many(key, read_duration)
    }

    pub fn size(&self, key: &str) -> Result<u64, String> {
        self.one(key, read_size)
    }

    pub fn sizes(&self, key: &str) -> Result<Vec<u64>, String> {
        self.many(key, read_size)
    }
}

fn wrong(key: &str, line: usize, want: &str, got: &Value) -> String {
    format!("line {}: '{}' is {}, not {}", line, key, got.kind(), want)
}

fn read_resolved(key: &str, value: &Value, line: usize) -> Result<Resolved, String> {
    Ok(match value {
        Value::Bool(b) => Resolved::Bool(*b),
        Value::Int(n) => Resolved::Int(*n),
        Value::Float(f) => Resolved::Float(*f),
        Value::Str(_) => Resolved::Str(read_str(key, value, line)?.to_string()),
        Value::Port(p) => Resolved::Port(*p),
        Value::Duration(..) => Resolved::Duration(read_duration(key, value, line)?),
        Value::Size(..) => Resolved::Size(read_size(key, value, line)?),
        Value::Colour(_) => Resolved::Colour(read_colour(key, value, line)?),
        Value::List(items) => Resolved::List(
            items
                .iter()
                .enumerate()
                .map(|(i, item)| read_resolved(&format!("{}[{}]", key, i), item, line))
                .collect::<Result<_, _>>()?,
        ),
        Value::Ref(_) => unreachable!("resolved values have no refs"),
    })
}

fn read_str<'a>(key: &str, value: &'a Value, line: usize) -> Result<&'a str, String> {
    match value {
        Value::Str(parts) => match parts.as_slice() {
            [Part::Text(t)] => Ok(t),
            _ => Err(format!("'{}' wasn't fully resolved", key)),
        },
        v => Err(wrong(key, line, "a string", v)),
    }
}

fn read_port(key: &str, value: &Value, line: usize) -> Result<u16, String> {
    match value {
        Value::Port(p) => Ok(*p),
        v => Err(wrong(key, line, "a port", v)),
    }
}

fn read_int(key: &str, value: &Value, line: usize) -> Result<i64, String> {
    match value {
        Value::Int(n) => Ok(*n),
        v => Err(wrong(key, line, "an int", v)),
    }
}

fn read_float(key: &str, value: &Value, line: usize) -> Result<f64, String> {
    match value {
        Value::Float(f) => Ok(*f),
        v => Err(wrong(key, line, "a float", v)),
    }
}

fn read_colour(key: &str, value: &Value, line: usize) -> Result<String, String> {
    match value {
        Value::Colour(hex) => Ok(format!("#{}", hex)),
        v => Err(wrong(key, line, "a colour", v)),
    }
}

fn read_bool(key: &str, value: &Value, line: usize) -> Result<bool, String> {
    match value {
        Value::Bool(b) => Ok(*b),
        v => Err(wrong(key, line, "a bool", v)),
    }
}

fn read_duration(key: &str, value: &Value, line: usize) -> Result<Duration, String> {
    match value {
        Value::Duration(n, unit) => {
            let secs = n * match unit.as_str() {
                "sec" => 1.0,
                "min" => 60.0,
                "hr" => 3600.0,
                "d" => 86400.0,
                "wk" => 604_800.0,
                "mo" => 30.0 * 86400.0,
                "yr" => 365.0 * 86400.0,
                _ => unreachable!("parser only produces known units"),
            };
            Duration::try_from_secs_f64(secs)
                .map_err(|_| format!("line {}: duration is out of range: '{}'", line, key))
        }
        v => Err(wrong(key, line, "a duration", v)),
    }
}

fn read_size(key: &str, value: &Value, line: usize) -> Result<u64, String> {
    match value {
        Value::Size(n, unit) => {
            let bytes = n * match unit.as_str() {
                "B" => 1.0,
                "kB" | "KB" => 1e3,
                "MB" => 1e6,
                "GB" => 1e9,
                "TB" => 1e12,
                "KiB" => 1024.0,
                "MiB" => 1024f64.powi(2),
                "GiB" => 1024f64.powi(3),
                "TiB" => 1024f64.powi(4),
                _ => unreachable!("parser only produces known units"),
            };
            let rounded = bytes.round();
            if (bytes - rounded).abs() > bytes * 4.0 * f64::EPSILON || rounded >= u64::MAX as f64 {
                return Err(format!(
                    "line {}: '{}' isn't a whole number of bytes that fits in u64",
                    line, key
                ));
            }
            Ok(rounded as u64)
        }
        v => Err(wrong(key, line, "a size", v)),
    }
}
