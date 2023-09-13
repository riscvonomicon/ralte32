// examples/test-rv32.rs
#![no_std]
#![no_main]

use ralte32::{define_tests, assert_eq};

fn test_multiplication() {
    assert_eq!(6 * 7, 42);
}

fn test_remainder() {
    assert_eq!(7 % 6, 1);
}

define_tests! {
    test_multiplication,
    test_remainder,
}
