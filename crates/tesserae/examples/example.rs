use std::env;
use std::path::Path;

fn main() -> Result<(), tesserae::Error> {
    // this is just the location of example.tess
    // CARGO_MANIFEST_DIR is the folder where this crate's Cargo.toml is
    let file = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("file")
        .join("example.tess");

    // tesserae::load loads the file from a path with any env
    // tesserae::load(Path::new("example.tess"), None)             loads with base variables
    // tesserae::load(Path::new("example.tess"), Some("prod"))     loads with @prod variables
    let tessera = env::var("TESSERAE")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "prod".to_string());
    let config = tesserae::load(&file, Some(&tessera))?;

    println!(
        "serving {} on port {}",
        config.str("url")?,
        config.port("port")?
    );
    println!(
        "request timeout: {:.0}s",
        config.duration("timeout")?.as_secs_f64()
    );
    println!("cache size: {} MiB", config.size("cache")? / 1024u64.pow(2));

    Ok(())
}
