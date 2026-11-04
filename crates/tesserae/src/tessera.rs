use crate::Parsed;
use crate::Value;
use std::collections::HashMap;
use std::mem::discriminant;
use strsim::damerau_levenshtein;

pub fn apply(parsed: Parsed, env: Option<&str>) -> Result<HashMap<String, (Value, usize)>, String> {
    let Parsed { entries, tesserae } = parsed;
    let mut map = HashMap::new();

    if let Some(env) = env
        && !tesserae.contains(env)
    {
        let suggestion = tesserae
            .iter()
            .map(|t| (damerau_levenshtein(env, t), t.as_str()))
            .filter(|(d, _)| *d <= 2)
            .min_by_key(|(d, t)| (*d, *t));

        return Err(match suggestion {
            Some((_, t)) => format!("no tessera matching '{}'. did you mean '{}'?", env, t),
            None => format!("no tessera matching '{}'.", env),
        });
    }

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
        let mut seen: HashMap<&str, usize> = HashMap::new();

        for e in entries.iter().filter(|e| e.tess.as_deref() == Some(env)) {
            if let Some(prev) = seen.insert(e.key.as_str(), e.line) {
                return Err(format!(
                    "line {}: '{}' already set in @{} on line {}",
                    e.line, e.key, env, prev
                ));
            }

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
    }

    Ok(map)
}
