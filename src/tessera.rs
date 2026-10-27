use crate::Entry;
use crate::Value;
use std::collections::HashMap;

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
