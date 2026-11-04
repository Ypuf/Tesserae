#![allow(dead_code)] // idk why it thinks the code is dead

use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use std::sync::atomic::{AtomicUsize, Ordering};

use tesserae::{Config, load_str};

static NEXT: AtomicUsize = AtomicUsize::new(0);

pub struct TempFile(PathBuf);

impl TempFile {
    pub fn new(src: &str) -> Self {
        let id = NEXT.fetch_add(1, Ordering::Relaxed);
        let path =
            Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("{}-{}.tess", process::id(), id));
        fs::write(&path, src).expect("couldn't write test config");
        TempFile(path)
    }

    pub fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

pub fn ok(src: &str) -> Config {
    load_str(src, None).unwrap_or_else(|e| panic!("expected {src:?} to load, got: {e}"))
}

pub fn ok_env(src: &str, env: &str) -> Config {
    load_str(src, Some(env)).unwrap_or_else(|e| panic!("expected {src:?} to load, got: {e}"))
}

pub fn err(src: &str) -> String {
    match load_str(src, None) {
        Ok(_) => panic!("expected {src:?} to fail to load"),
        Err(e) => e,
    }
}

pub fn err_env(src: &str, env: &str) -> String {
    match load_str(src, Some(env)) {
        Ok(_) => panic!("expected {src:?} to fail to load under {env}"),
        Err(e) => e,
    }
}
