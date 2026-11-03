use crate::{Part, Value};
use std::collections::HashMap;
use std::time::Duration;

pub struct Config {
    map: HashMap<String, (Value, usize)>,
}

impl Config {
    pub(crate) fn new(map: HashMap<String, (Value, usize)>) -> Self {
        Config { map }
    }
    pub fn raw(&self) -> &HashMap<String, (Value, usize)> {
        &self.map
    }

    fn get(&self, key: &str) -> Result<&(Value, usize), String> {
        self.map
            .get(key)
            .ok_or_else(|| format!("no key '{}' in config", key))
    }

    fn wrong(key: &str, line: usize, want: &str, got: &Value) -> String {
        format!("line {}: '{}' is {}, not {}", line, key, kind(got), want)
    }

    pub fn str(&self, key: &str) -> Result<&str, String> {
        match self.get(key)? {
            (Value::Str(parts), _) => match parts.as_slice() {
                [Part::Text(t)] => Ok(t),
                _ => Err(format!("'{}' wasn't fully resolved", key)),
            },
            (v, line) => Err(Self::wrong(key, *line, "a string", v)),
        }
    }

    pub fn port(&self, key: &str) -> Result<u16, String> {
        match self.get(key)? {
            (Value::Port(p), _) => Ok(*p),
            (v, line) => Err(Self::wrong(key, *line, "a port", v)),
        }
    }

    pub fn int(&self, key: &str) -> Result<i64, String> {
        match self.get(key)? {
            (Value::Int(n), _) => Ok(*n),
            (v, line) => Err(Self::wrong(key, *line, "an int", v)),
        }
    }

    pub fn float(&self, key: &str) -> Result<f64, String> {
        match self.get(key)? {
            (Value::Float(f), _) => Ok(*f),
            (v, line) => Err(Self::wrong(key, *line, "a float", v)),
        }
    }

    pub fn colour(&self, key: &str) -> Result<String, String> {
        match self.get(key)? {
            (Value::Colour(hex), _) => Ok(format!("#{}", hex)),
            (v, line) => Err(Self::wrong(key, *line, "a colour", v)),
        }
    }

    pub fn bool(&self, key: &str) -> Result<bool, String> {
        match self.get(key)? {
            (Value::Bool(b), _) => Ok(*b),
            (v, line) => Err(Self::wrong(key, *line, "a bool", v)),
        }
    }

    pub fn duration(&self, key: &str) -> Result<Duration, String> {
        match self.get(key)? {
            (Value::Duration(n, unit), line) => {
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
            (v, line) => Err(Self::wrong(key, *line, "a duration", v)),
        }
    }

    pub fn size(&self, key: &str) -> Result<u64, String> {
        match self.get(key)? {
            (Value::Size(n, unit), line) => {
                let bits = n * match unit.as_str() {
                    "b" => 1.0,
                    "B" => 8.0,
                    "kB" => 8e3,
                    "MB" => 8e6,
                    "GB" => 8e9,
                    "TB" => 8e12,
                    "KiB" => 8.0 * 1024.0,
                    "MiB" => 8.0 * 1024f64.powi(2),
                    "GiB" => 8.0 * 1024f64.powi(3),
                    "TiB" => 8.0 * 1024f64.powi(4),
                    _ => unreachable!("parser only produces known units"),
                };
                let rounded = bits.round();
                if (bits - rounded).abs() > bits * 4.0 * f64::EPSILON
                    || rounded >= u64::MAX as f64
                {
                    return Err(format!(
                        "line {}: '{}' isn't a whole number of bits that fits in u64",
                        line, key
                    ));
                }
                Ok(rounded as u64)
            }
            (v, line) => Err(Self::wrong(key, *line, "a size", v)),
        }
    }
}

fn kind(v: &Value) -> &'static str {
    match v {
        Value::Bool(_) => "a bool",
        Value::Int(_) => "an int",
        Value::Float(_) => "a float",
        Value::Str(_) | Value::Ref(_) => "a string",
        Value::Port(_) => "a port",
        Value::Duration(..) => "a duration",
        Value::Size(..) => "a size",
        Value::Colour(_) => "a colour",
    }
}
