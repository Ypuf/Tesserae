use std::collections::HashMap;
use std::path::Path;

mod parse;
mod resolve;
mod tessera;
mod value;

pub use parse::{Entry, Parsed, parse};
pub use resolve::resolve;
pub use tessera::apply;
pub use value::{Part, Value};

pub fn load(path: &Path, env: Option<&str>) -> Result<HashMap<String, (Value, usize)>, String> {
    resolve(apply(parse(path)?, env)?)
}
