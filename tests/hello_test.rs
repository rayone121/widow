use widow_lib::run_hello;

#[test]
fn test_hello_world() {
    // Run the hello world program
    let result = run_hello("examples/hello.wd");
    assert!(result.is_ok(), "Failed to run hello.wd: {:?}", result.err());
}