#![allow(clippy::unnecessary_literal_unwrap)]
#![allow(clippy::field_reassign_with_default)]
#![allow(clippy::bool_assert_comparison)]
#![allow(unused)]
#![allow(clippy::uninlined_format_args)]
use zpm::error::ZpmError;

#[test]
fn error_display_variants() {
    let e = ZpmError::UnsupportedCommand {
        agent: "npm".to_string(),
        command: "dedupe".to_string(),
    };
    assert!(e.to_string().contains("unsupported"));
    assert!(e.to_string().contains("npm"));
    assert!(e.to_string().contains("dedupe"));

    let e = ZpmError::UnknownAgent("foo".to_string());
    assert!(e.to_string().contains("unknown agent"));
    assert!(e.to_string().contains("foo"));

    let e = ZpmError::Detection("failed".to_string());
    assert!(e.to_string().contains("detection failed"));

    let e = ZpmError::Ambiguous("multiple".to_string());
    assert!(e.to_string().contains("ambiguous"));

    let e = ZpmError::Config("bad config".to_string());
    assert!(e.to_string().contains("config error"));

    let e = ZpmError::Other("something".to_string());
    assert!(e.to_string().contains("something"));

    let e = ZpmError::CommandFailed {
        program: "npm".to_string(),
        args: "install".to_string(),
        code: Some(1),
    };
    assert!(e.to_string().contains("exit code"));
    assert!(e.to_string().contains("npm"));
    let e2 = ZpmError::CommandFailed {
        program: "npm".to_string(),
        args: "".to_string(),
        code: None,
    };
    assert!(e2.to_string().contains("exit code"));

    // Io and Json from traits
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let e: ZpmError = io_err.into();
    assert!(e.to_string().contains("IO error"));

    let json_err = serde_json::from_str::<serde_json::Value>("not json").unwrap_err();
    let e: ZpmError = json_err.into();
    assert!(e.to_string().contains("JSON error"));
}

#[test]
fn error_from_conversions() {
    // Ensure From impls work
    let io = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
    let z: ZpmError = ZpmError::from(io);
    assert!(matches!(z, ZpmError::Io(_)));

    let json = serde_json::from_str::<serde_json::Value>("{").unwrap_err();
    let z: ZpmError = ZpmError::from(json);
    assert!(matches!(z, ZpmError::Json(_)));
}

#[test]
fn result_type_alias() {
    let r: zpm::Result<i32> = Ok(42);
    assert!(r.is_ok());
    assert_eq!(r.unwrap_or(0), 42);
    let e: zpm::Result<i32> = Err(ZpmError::Other("err".to_string()));
    assert!(e.is_err());
}
