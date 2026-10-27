mod parse;
mod tessera;
mod value;

pub use parse::{Entry, parse};
pub use tessera::apply;
pub use value::{Part, Value};
