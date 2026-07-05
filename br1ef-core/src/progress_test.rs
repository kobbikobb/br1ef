#[test]
fn with_progress_returns_value() {
    let result = super::with_progress("test", || 42);
    assert_eq!(result, 42);
}

#[test]
fn with_progress_passes_through_error() {
    let result: Result<i32, &str> = super::with_progress("test", || Err("nope"));
    assert_eq!(result, Err("nope"));
}
