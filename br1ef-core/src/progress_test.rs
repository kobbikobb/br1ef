use super::estimate_duration;
use std::time::Duration;

#[test]
fn estimates_duration_from_prompt_size() {
    let d = estimate_duration("a".repeat(100).as_str());
    assert!(d >= Duration::from_millis(200));
    assert!(d <= Duration::from_millis(5_000));
}

#[test]
fn tiny_prompt_gets_minimum_duration() {
    let d = estimate_duration("");
    assert_eq!(d, Duration::from_millis(200));
}

#[test]
fn huge_prompt_gets_maximum_duration() {
    let d = estimate_duration(&"x".repeat(100_000));
    assert_eq!(d, Duration::from_millis(5_000));
}

#[test]
fn linear_scaling() {
    let small = estimate_duration(&"x".repeat(100));
    let big = estimate_duration(&"x".repeat(1000));
    assert!(
        big > small,
        "bigger prompt should get longer estimate: {big:?} <= {small:?}",
    );
}

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
