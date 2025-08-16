use betfair_rs::config::{BetfairConfig, Config};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_config_new_with_valid_file() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("config.toml");

    let config_content = r#"
[betfair]
username = "test_user"
password = "test_pass"
api_key = "test_key"
pfx_path = "/path/to/cert.pfx"
pfx_password = "cert_pass"
"#;

    fs::write(&config_path, config_content).unwrap();

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(dir.path()).unwrap();

    let result = Config::new();

    std::env::set_current_dir(original_dir).unwrap();

    assert!(result.is_ok());
    let config = result.unwrap();
    assert_eq!(config.betfair.username, "test_user");
    assert_eq!(config.betfair.password, "test_pass");
    assert_eq!(config.betfair.api_key, "test_key");
    assert_eq!(config.betfair.pfx_path, "/path/to/cert.pfx");
    assert_eq!(config.betfair.pfx_password, "cert_pass");
}

#[test]
fn test_config_new_with_missing_file() {
    let dir = tempdir().unwrap();

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(dir.path()).unwrap();

    let result = Config::new();

    std::env::set_current_dir(original_dir).unwrap();

    assert!(result.is_err());
}

#[test]
fn test_config_new_with_invalid_toml() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("config.toml");

    let invalid_content = r#"
[betfair
username = "test_user"
"#;

    fs::write(&config_path, invalid_content).unwrap();

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(dir.path()).unwrap();

    let result = Config::new();

    std::env::set_current_dir(original_dir).unwrap();

    assert!(result.is_err());
}

#[test]
fn test_config_new_with_missing_fields() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("config.toml");

    let incomplete_content = r#"
[betfair]
username = "test_user"
"#;

    fs::write(&config_path, incomplete_content).unwrap();

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(dir.path()).unwrap();

    let result = Config::new();

    std::env::set_current_dir(original_dir).unwrap();

    assert!(result.is_err());
}

#[test]
fn test_betfair_config_clone() {
    let config = BetfairConfig {
        username: "user".to_string(),
        password: "pass".to_string(),
        api_key: "key".to_string(),
        pfx_path: "/path".to_string(),
        pfx_password: "pfx_pass".to_string(),
    };

    let cloned = config.clone();
    assert_eq!(config.username, cloned.username);
    assert_eq!(config.password, cloned.password);
    assert_eq!(config.api_key, cloned.api_key);
    assert_eq!(config.pfx_path, cloned.pfx_path);
    assert_eq!(config.pfx_password, cloned.pfx_password);
}

#[test]
fn test_config_clone() {
    let betfair_config = BetfairConfig {
        username: "user".to_string(),
        password: "pass".to_string(),
        api_key: "key".to_string(),
        pfx_path: "/path".to_string(),
        pfx_password: "pfx_pass".to_string(),
    };

    let config = Config {
        betfair: betfair_config,
    };

    let cloned = config.clone();
    assert_eq!(config.betfair.username, cloned.betfair.username);
    assert_eq!(config.betfair.api_key, cloned.betfair.api_key);
}
