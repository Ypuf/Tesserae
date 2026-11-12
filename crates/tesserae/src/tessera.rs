use crate::Parsed;
use crate::Value;
use crate::error::{Error, ErrorKind};
use std::collections::{HashMap, HashSet};
use std::mem::discriminant;
use strsim::damerau_levenshtein;

pub(crate) fn apply(
    parsed: Parsed,
    env: Option<&str>,
) -> Result<HashMap<String, (Value, usize)>, Error> {
    let Parsed { entries, tesserae } = parsed;
    let mut map = HashMap::new();

    if let Some(env) = env
        && !tesserae.contains(env)
    {
        let suggestion = tesserae
            .iter()
            .map(|t| (damerau_levenshtein(env, t), t.as_str()))
            .filter(|(d, _)| *d <= 2)
            .min_by_key(|(d, t)| (*d, *t))
            .map(|(_, t)| t.to_string());

        return Err(Error::new(ErrorKind::UnknownTessera {
            name: env.to_string(),
            suggestion,
        }));
    }

    for e in entries.iter().filter(|e| e.tess.is_none()) {
        if let Some((_, prev)) = map.get(&e.key) {
            return Err(Error::at(
                e.line,
                ErrorKind::DuplicateKey {
                    key: e.key.clone(),
                    first_line: *prev,
                },
            ));
        }
        map.insert(e.key.clone(), (e.value.clone(), e.line));
    }

    check_names(&map, &tesserae)?;

    if let Some(env) = env {
        let mut seen: HashMap<&str, usize> = HashMap::new();

        for e in entries.iter().filter(|e| e.tess.as_deref() == Some(env)) {
            if let Some(prev) = seen.insert(e.key.as_str(), e.line) {
                return Err(Error::at(
                    e.line,
                    ErrorKind::DuplicateOverride {
                        tessera: env.to_string(),
                        key: e.key.clone(),
                        first_line: prev,
                    },
                ));
            }

            let Some((base_value, base_line)) = map.get(&e.key) else {
                return Err(Error::at(
                    e.line,
                    ErrorKind::OverrideNotInBase {
                        tessera: env.to_string(),
                        key: e.key.clone(),
                    },
                ));
            };

            let either_ref =
                matches!(base_value, Value::Ref(_)) || matches!(e.value, Value::Ref(_));
            if !either_ref && discriminant(base_value) != discriminant(&e.value) {
                return Err(Error::at(
                    e.line,
                    ErrorKind::OverrideTypeMismatch {
                        tessera: env.to_string(),
                        key: e.key.clone(),
                        base_line: *base_line,
                    },
                ));
            }

            map.insert(e.key.clone(), (e.value.clone(), e.line));
        }
    }

    Ok(map)
}

fn check_names(
    map: &HashMap<String, (Value, usize)>,
    tesserae: &HashSet<String>,
) -> Result<(), Error> {
    let mut keys: Vec<&String> = map.keys().collect();
    keys.sort();

    for child in &keys {
        let child_line = map[*child].1;
        for (i, _) in child.match_indices('/') {
            let key = &child[..i];
            if let Some((_, value_line)) = map.get(key) {
                return Err(Error::at(
                    child_line.max(*value_line),
                    ErrorKind::ValueAndSection {
                        key: key.to_string(),
                        value_line: *value_line,
                        child: child.to_string(),
                        child_line,
                    },
                ));
            }
        }
    }

    for key in keys {
        if !key.contains('/') && tesserae.contains(key) {
            return Err(Error::at(
                map[key].1,
                ErrorKind::KeyAndTessera { key: key.clone() },
            ));
        }
    }

    Ok(())
}
