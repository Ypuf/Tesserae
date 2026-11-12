use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

use crate::ValueKind;

#[derive(Debug)]
pub struct Error(Box<Inner>);

#[derive(Debug)]
struct Inner {
    kind: ErrorKind,
    path: Option<PathBuf>,
    line: Option<usize>,
    tessera: Option<String>,
    source: Option<io::Error>,
}

impl Error {
    pub(crate) fn new(kind: ErrorKind) -> Self {
        Error(Box::new(Inner {
            kind,
            path: None,
            line: None,
            tessera: None,
            source: None,
        }))
    }

    pub(crate) fn at(line: usize, kind: ErrorKind) -> Self {
        let mut error = Error::new(kind);
        error.0.line = Some(line);
        error
    }

    pub(crate) fn read(source: io::Error) -> Self {
        let mut error = Error::new(ErrorKind::Read {
            error: source.kind(),
        });
        error.0.source = Some(source);
        error
    }

    pub(crate) fn in_file(mut self, path: Option<&Path>) -> Self {
        self.0.path = path.map(Path::to_path_buf);
        self
    }

    pub(crate) fn in_tessera(mut self, name: &str) -> Self {
        self.0.tessera = Some(name.to_string());
        self
    }

    pub fn kind(&self) -> &ErrorKind {
        &self.0.kind
    }

    pub fn path(&self) -> Option<&Path> {
        self.0.path.as_deref()
    }

    pub fn line(&self) -> Option<usize> {
        self.0.line
    }

    pub fn tessera(&self) -> Option<&str> {
        match &self.0.kind {
            ErrorKind::DuplicateOverride { tessera, .. }
            | ErrorKind::OverrideNotInBase { tessera, .. }
            | ErrorKind::OverrideTypeMismatch { tessera, .. }
            | ErrorKind::ResolvedTypeMismatch { tessera, .. } => Some(tessera),
            _ => self.0.tessera.as_deref(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(path) = &self.0.path {
            write!(f, "{}: ", path.display())?;
        }
        if let Some(tessera) = &self.0.tessera {
            write!(f, "@{}: ", tessera)?;
        }
        if let Some(line) = self.0.line {
            write!(f, "line {}: ", line)?;
        }
        write!(f, "{}", self.0.kind)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.0.source.as_ref().map(|e| e as _)
    }
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        self.0.kind == other.0.kind
            && self.0.path == other.0.path
            && self.0.line == other.0.line
            && self.0.tessera == other.0.tessera
    }
}

impl Eq for Error {}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ErrorKind {
    Read {
        error: io::ErrorKind,
    },

    InvalidTesseraName {
        text: String,
    },
    InvalidPath {
        text: String,
    },
    EmptyKey {
        text: String,
    },
    InvalidKey {
        text: String,
    },
    InvalidLine {
        text: String,
    },

    BadEscape {
        string: String,
    },
    UnclosedReference {
        string: String,
    },
    EmptyReference {
        string: String,
    },
    UnmatchedBrace {
        string: String,
    },
    UnescapedQuote {
        string: String,
    },
    UnterminatedString {
        value: String,
    },
    UnterminatedList {
        list: String,
    },
    EmptyListItem {
        list: String,
    },
    NestedList {
        list: Option<String>,
    },
    InvalidPort {
        value: String,
    },
    InvalidColour {
        value: String,
    },
    FloatOutOfRange {
        value: String,
    },
    UnknownValue {
        value: String,
    },

    UnknownTessera {
        name: String,
        suggestion: Option<String>,
    },

    DuplicateKey {
        key: String,
        first_line: usize,
    },
    ValueAndSection {
        key: String,
        value_line: usize,
        child: String,
        child_line: usize,
    },
    KeyAndTessera {
        key: String,
    },
    DuplicateOverride {
        tessera: String,
        key: String,
        first_line: usize,
    },
    OverrideNotInBase {
        tessera: String,
        key: String,
    },
    OverrideTypeMismatch {
        tessera: String,
        key: String,
        base_line: usize,
    },
    ResolvedTypeMismatch {
        tessera: String,
        key: String,
        base_line: usize,
    },

    UnknownReference {
        reference: String,
    },
    ReferenceCycle {
        cycle: Vec<String>,
    },
    ListInString {
        reference: String,
    },
    MixedList {
        first: ValueKind,
        other: ValueKind,
    },

    MissingKey {
        key: String,
    },
    WrongType {
        key: String,
        expected: ValueKind,
        found: ValueKind,
    },
    DurationOutOfRange {
        key: String,
    },
    InvalidSize {
        key: String,
    },
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorKind::Read { .. } => write!(f, "couldn't read file"),

            ErrorKind::InvalidTesseraName { text } => {
                write!(f, "tessera name is invalid: '{}'", text)
            }
            ErrorKind::InvalidPath { text } => write!(f, "path name is invalid: '{}'", text),
            ErrorKind::EmptyKey { text } => write!(f, "key can't be empty: '{}'", text),
            ErrorKind::InvalidKey { text } => write!(f, "key name is invalid: '{}'", text),
            ErrorKind::InvalidLine { text } => write!(f, "can't parse {:?}", text),

            ErrorKind::BadEscape { string } => write!(f, "bad escape in \"{}\"", string),
            ErrorKind::UnclosedReference { string } => write!(f, "unclosed {{ in \"{}\"", string),
            ErrorKind::EmptyReference { string } => write!(f, "empty reference in \"{}\"", string),
            ErrorKind::UnmatchedBrace { string } => write!(f, "unmatched }} in \"{}\"", string),
            ErrorKind::UnescapedQuote { string } => write!(f, "unescaped quote in \"{}\"", string),
            ErrorKind::UnterminatedString { value } => {
                write!(f, "unterminated string: {}", value)
            }
            ErrorKind::UnterminatedList { list } => write!(f, "unterminated list: {}", list),
            ErrorKind::EmptyListItem { list } => write!(f, "empty item in list: {}", list),
            ErrorKind::NestedList { list: Some(list) } => {
                write!(f, "lists can't be nested: {}", list)
            }
            ErrorKind::NestedList { list: None } => write!(f, "lists can't be nested"),
            ErrorKind::InvalidPort { value } => write!(f, "invalid port: '{}'", value),
            ErrorKind::InvalidColour { value } => write!(f, "invalid colour: '{}'", value),
            ErrorKind::FloatOutOfRange { value } => write!(f, "float is out of range: '{}'", value),
            ErrorKind::UnknownValue { value } => write!(f, "unknown value: '{}'", value),

            ErrorKind::UnknownTessera {
                name,
                suggestion: Some(suggestion),
            } => write!(
                f,
                "no tessera matching '{}'. did you mean '{}'?",
                name, suggestion
            ),
            ErrorKind::UnknownTessera {
                name,
                suggestion: None,
            } => write!(f, "no tessera matching '{}'.", name),
            ErrorKind::DuplicateKey { key, first_line } => {
                write!(f, "'{}' already defined on line {}", key, first_line)
            }
            ErrorKind::ValueAndSection {
                key,
                value_line,
                child,
                child_line,
            } => write!(
                f,
                "'{}' is both a value (line {}) and a section ('{}' on line {})",
                key, value_line, child, child_line
            ),
            ErrorKind::KeyAndTessera { key } => {
                write!(f, "'{}' is both a key and a tessera (@{})", key, key)
            }
            ErrorKind::DuplicateOverride {
                tessera,
                key,
                first_line,
            } => write!(
                f,
                "'{}' already set in @{} on line {}",
                key, tessera, first_line
            ),
            ErrorKind::OverrideNotInBase { tessera, key } => {
                write!(f, "@{} overrides '{}', which isn't in base", tessera, key)
            }
            ErrorKind::OverrideTypeMismatch {
                tessera,
                key,
                base_line,
            } => write!(
                f,
                "@{} sets '{}' to a different type than base (line {})",
                tessera, key, base_line
            ),
            ErrorKind::ResolvedTypeMismatch {
                tessera,
                key,
                base_line,
            } => write!(
                f,
                "@{} resolves '{}' to a different type than base (line {})",
                tessera, key, base_line
            ),

            ErrorKind::UnknownReference { reference } => {
                write!(f, "reference to '{}', which doesn't exist", reference)
            }
            ErrorKind::ReferenceCycle { cycle } => {
                write!(f, "reference cycle: {}", cycle.join(" -> "))
            }
            ErrorKind::ListInString { reference } => {
                write!(f, "can't put list '{}' inside a string", reference)
            }
            ErrorKind::MixedList { first, other } => {
                write!(f, "list mixes {} and {}", first, other)
            }

            ErrorKind::MissingKey { key } => write!(f, "no key '{}' in config", key),
            ErrorKind::WrongType {
                key,
                expected,
                found,
            } => write!(f, "'{}' is {}, not {}", key, found, expected),
            ErrorKind::DurationOutOfRange { key } => {
                write!(f, "duration is out of range: '{}'", key)
            }
            ErrorKind::InvalidSize { key } => {
                write!(
                    f,
                    "'{}' isn't a whole number of bytes that fits in u64",
                    key
                )
            }
        }
    }
}
