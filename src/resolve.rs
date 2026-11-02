use crate::{Part, Value};
use std::collections::HashMap;

type Map = HashMap<String, (Value, usize)>;

pub fn resolve(map: Map) -> Result<Map, String> {
    let mut keys: Vec<&String> = map.keys().collect();
    keys.sort();

    let mut done = HashMap::new();
    let mut stack = Vec::new();

    for key in keys {
        reach(key, &map, &mut done, &mut stack)?;
    }

    Ok(done)
}

fn reach<'a>(
    key: &'a str,
    map: &'a Map,
    done: &mut Map,
    stack: &mut Vec<&'a str>,
) -> Result<Value, String> {
    if let Some((value, _)) = done.get(key) {
        return Ok(value.clone());
    }

    let (value, line) = &map[key];
    stack.push(key);

    let resolved = match value {
        Value::Ref(path) => stalk(path, *line, map, done, stack)?,
        Value::Str(parts) => {
            let mut text = String::new();
            for part in parts {
                match part {
                    Part::Text(t) => text.push_str(t),
                    Part::Ref(path) => {
                        text.push_str(&strify(&stalk(path, *line, map, done, stack)?))
                    }
                }
            }
            Value::Str(vec![Part::Text(text)])
        }
        other => other.clone(),
    };

    stack.pop();
    done.insert(key.to_string(), (resolved.clone(), *line));
    Ok(resolved)
}

fn stalk<'a>(
    path: &'a str,
    line: usize,
    map: &'a Map,
    done: &mut Map,
    stack: &mut Vec<&'a str>,
) -> Result<Value, String> {
    if let Some(start) = stack.iter().position(|k| *k == path) {
        let mut cycle = stack[start..].to_vec();
        cycle.push(path);
        return Err(format!(
            "line {}: reference cycle: {}",
            line,
            cycle.join(" -> ")
        ));
    }

    if !map.contains_key(path) {
        return Err(format!(
            "line {}: reference to '{}', which doesn't exist",
            line, path
        ));
    }

    reach(path, map, done, stack)
}

fn strify(value: &Value) -> String {
    match value {
        Value::Bool(b) => b.to_string(),
        Value::Int(n) => n.to_string(),
        Value::Float(f) => format!("{:?}", f),
        Value::Str(parts) => parts
            .iter()
            .map(|part| match part {
                Part::Text(t) => t.as_str(),
                Part::Ref(_) => unreachable!("resolved strings have no refs"),
            })
            .collect(),
        Value::Port(p) => p.to_string(),
        Value::Duration(n, unit) | Value::Size(n, unit) => format!("{}{}", n, unit),
        Value::Colour(hex) => format!("#{}", hex),
        Value::Ref(_) => unreachable!("resolved values have no refs"),
    }
}
