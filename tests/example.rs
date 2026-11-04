use std::collections::BTreeMap;
use std::path::Path;
use std::time::Duration;

use tesserae::{Config, load, load_str};

fn example(env: Option<&str>) -> Config {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("file/example.tess");
    load(&path, env).unwrap_or_else(|e| panic!("example failed under {env:?}: {e}"))
}

#[test]
fn base() {
    let config = example(None);
    assert_eq!(config.str("url"), Ok("http://localhost:5432/api"));
    assert_eq!(config.str("host"), Ok("localhost"));
    assert_eq!(config.port("port"), Ok(5432));
    assert_eq!(config.str("proc"), Ok("http"));
    assert_eq!(config.duration("timeout"), Ok(Duration::from_secs(30)));
    assert_eq!(config.str("db/host"), Ok("localhost"));
    assert_eq!(config.port("db/port"), Ok(5433));
    assert_eq!(config.colour("colour").as_deref(), Ok("#FF6611"));
}

#[test]
fn prod() {
    let config = example(Some("prod"));
    assert_eq!(config.str("url"), Ok("https://ypuf.xyz:25565/api"));
    assert_eq!(config.str("host"), Ok("ypuf.xyz"));
    assert_eq!(config.port("port"), Ok(25565));
    assert_eq!(config.str("proc"), Ok("https"));
    assert_eq!(config.duration("timeout"), Ok(Duration::from_secs(30)));
    assert_eq!(config.str("db/host"), Ok("db.ypuf.xyz"));
    assert_eq!(config.port("db/port"), Ok(25566));
    assert_eq!(config.colour("colour").as_deref(), Ok("#FF6611"));
}

fn dump(config: &Config) -> String {
    format!("{:?}", config.raw().iter().collect::<BTreeMap<_, _>>())
}

#[test]
fn load_str_matches_load() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("file/example.tess");
    let src = std::fs::read_to_string(&path).unwrap();

    for env in [None, Some("prod")] {
        let from_str = load_str(&src, env).unwrap();
        assert_eq!(dump(&from_str), dump(&example(env)), "{env:?}");
    }
}
