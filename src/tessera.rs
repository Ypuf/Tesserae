use crate::Entry;
use crate::Value;
use std::collections::HashMap;
use std::mem::discriminant;

pub fn apply(
    entries: Vec<Entry>,
    env: Option<&str>,
) -> Result<HashMap<String, (Value, usize)>, String> {
    let mut map = HashMap::new();

    for e in entries.iter().filter(|e| e.tess.is_none()) {
        if let Some((_, prev)) = map.get(&e.key) {
            return Err(format!(
                "line {}: '{}' already defined on line {}",
                e.line, e.key, prev
            ));
        }
        map.insert(e.key.clone(), (e.value.clone(), e.line));
    }

    if let Some(env) = env {
        for e in entries.iter().filter(|e| e.tess.as_deref() == Some(env)) {
            let Some((base_value, base_line)) = map.get(&e.key) else {
                return Err(format!(
                    "line {}: @{} overrides '{}', which isn't in base",
                    e.line, env, e.key
                ));
            };

            let either_ref =
                matches!(base_value, Value::Ref(_)) || matches!(e.value, Value::Ref(_));
            if !either_ref && discriminant(base_value) != discriminant(&e.value) {
                return Err(format!(
                    "line {}: @{} sets '{}' to a different type than base (line {})",
                    e.line, env, e.key, base_line
                ));
            }

            map.insert(e.key.clone(), (e.value.clone(), e.line));
        }

        for e in entries.iter().filter(|e| e.tess.as_deref() == Some(env)) {
            if !map.contains_key(&e.key) {
                return Err(format!(
                    "line {}: @{} overrides '{}', which isn't in base",
                    e.line, env, e.key
                ));
            }
            map.insert(e.key.clone(), (e.value.clone(), e.line));
        }
    }

    Ok(map)
}
