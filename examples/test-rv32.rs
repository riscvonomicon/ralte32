// examples/test-rv32.rs
#![cfg_attr(target_arch = "riscv32", no_std)]
#![cfg_attr(target_arch = "riscv32", no_main)]

use ralte32::{assert_eq, define_tests};

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
