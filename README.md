# RALTE32

A **R**ust **A**rithmetic **L**ibrary **T**esting **E**nvironment for embedded
RISC-V **32**-bit. This libraries allows the testing of arithmetic Rust code
made for [RISC-V] 32-bit using the [QEMU] simulator. This is especially useful
when developing with the [Rust riscv32 intrinsics].

This library is mostly just a minimal hack to implement a testing environment
and port the `riscv32` embedded targets to a Linux userspace target.

## Usage

First, this project uses the [QEMU] userspace simulator to simulate the target
code. This can be installed with the standard `qemu-user` package on most
operating systems.

```bash
# Linux: Debian / Ubuntu
sudo apt-get install qemu-user

# Linux: ArchLinux
sudo pacman -S qemu-user
```

For more platforms, take a look [here](https://www.qemu.org/download).

Then, add `ralte32` as a development dependency.

```bash
cargo add --dev ralte32
```

Lastly, create and/or add a short section to your `.cargo/config.toml`.

```toml
# ...

[target.riscv32imac-unknown-none-elf]
rustflags = ['-Ctarget-feature=+crt-static']
runner = "qemu-riscv32 -cpu rv32"

# NOTE: If you want to enable additional target features, add them here.
# 
# Example to enable the `zk` feature:
# rustflags = ['-Ctarget-feature=+crt-static,+zk']
# runner = "qemu-riscv32 -cpu rv32,zk=true"
```

Then, to implement some tests, you add an example in `examples/`.

```rust,no_run
// examples/test-rv32.rs
#![no_std]
#![no_main]

use ralte32::define_tests;

fn test_multiplication() {
    assert_eq!(6 * 7, 42);
}

fn test_remainder() {
    assert_eq!(7 % 6, 1);
}

define_tests!{
    test_multiplication,
    test_remainder,
};
```

This can then be ran with:

```bash
cargo run --example test-rv32 --target riscv32imac-unknown-none-elf
```

This will give:

```text
Running tests...

Running "test_multiplication"... SUCCESSFUL
Running "test_remainder"... SUCCESSFUL
```

## Limitations

There are several known limitations.

1. First test or assert to fail, stops the test environment.
2. This only tests user-level code. Access to supervisor, machine or hypervisor
   instructions and CSRs is not possible.
3. Very limited support for printing.

## License

This project is dual licensed under [MIT](./LICENSE-MIT) and
[APACHE-2.0](./LICENSE-APACHE) licenses.

[QEMU]: https://www.qemu.org/
[RISC-V]: https://en.wikipedia.org/wiki/RISC-V
[Rust riscv32 intrinsics]: https://doc.rust-lang.org/nightly/core/arch/riscv32