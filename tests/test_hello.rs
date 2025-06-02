use std::fs;
use widow_lib::{run_file};

#[test]
fn test_hello_world() {
    let result = run_file("examples/hello.wd");
    assert!(result.is_ok(), "Failed to run hello.wd: {:?}", result.err());
}