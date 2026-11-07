use crate::error::{Error, ErrorKind};
use crate::{Part, Value};
use std::collections::HashMap;
use std::mem::discriminant;

type Map = HashMap<String, (Value, usize)>;

pub(crate) fn resolve(map: Map) -> Result<Map, Error> {
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
) -> Result<Value, Error> {
    if let Some((value, _)) = done.get(key) {
        return Ok(value.clone());
    }

    let (value, line) = &map[key];
    stack.push(key);

    let resolved = settle(value, *line, map, done, stack)?;

    stack.pop();
    done.insert(key.to_string(), (resolved.clone(), *line));
    Ok(resolved)
}

fn settle<'a>(
    value: &'a Value,
    line: usize,
    map: &'a Map,
    done: &mut Map,
    stack: &mut Vec<&'a str>,
) -> Result<Value, Error> {
    match value {
        Value::Ref(path) => stalk(path, line, map, done, stack),
        Value::Str(parts) => {
            let mut text = String::new();
            for part in parts {
                match part {
                    Part::Text(t) => text.push_str(t),
                    Part::Ref(path) => {
                        let target = stalk(path, line, map, done, stack)?;
                        if matches!(target, Value::List(_)) {
                            return Err(Error::at(
                                line,
                                ErrorKind::ListInString {
                                    reference: path.clone(),
                                },
                            ));
                        }
                        text.push_str(&strify(&target));
                    }
                }
            }
            Ok(Value::Str(vec![Part::Text(text)]))
        }
        Value::List(items) => {
            let mut resolved = Vec::new();
            for item in items {
                resolved.push(settle(item, line, map, done, stack)?);
            }
            check_items(&resolved, line)?;
            Ok(Value::List(resolved))
        }
        other => Ok(other.clone()),
    }
}

fn check_items(items: &[Value], line: usize) -> Result<(), Error> {
    for item in items {
        if matches!(item, Value::List(_)) {
            return Err(Error::at(line, ErrorKind::NestedList { list: None }));
        }
        if discriminant(item) != discriminant(&items[0]) {
            return Err(Error::at(
                line,
                ErrorKind::MixedList {
                    first: items[0].kind(),
                    other: item.kind(),
                },
            ));
        }
    }
    Ok(())
}

fn stalk<'a>(
    path: &'a str,
    line: usize,
    map: &'a Map,
    done: &mut Map,
    stack: &mut Vec<&'a str>,
) -> Result<Value, Error> {
    if let Some(start) = stack.iter().position(|k| *k == path) {
        let mut cycle: Vec<String> = stack[start..].iter().map(|k| k.to_string()).collect();
        cycle.push(path.to_string());
        return Err(Error::at(line, ErrorKind::ReferenceCycle { cycle }));
    }

    if !map.contains_key(path) {
        return Err(Error::at(
            line,
            ErrorKind::UnknownReference {
                reference: path.to_string(),
            },
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
        Value::List(_) => unreachable!("lists are rejected before this"),
    }
}
