//! A **R**ust **A**rithmetic **L**ibrary **T**esting **E**nvironment for embedded RISC-V
//! **32**-bit. This libraries allows the testing of arithmetic Rust code made for [RISC-V] 32-bit
//! using the [QEMU] simulator. This is especially useful when developing with the [Rust riscv32
//! intrinsics].
//!
//! This library is mostly just a minimal hack to implement a testing environment and port the
//! `riscv32` embedded targets to a Linux userspace target.
//!
//! ## Usage
//!
//! First, this project uses the [QEMU] userspace simulator to simulate the target code. This can
//! be installed with the standard `qemu-user` package on most operating systems.
//!
//! ```bash
//! # Linux: Debian / Ubuntu
//! sudo apt-get install qemu-user
//!
//! # Linux: ArchLinux
//! sudo pacman -S qemu-user
//! ```
//!
//! For more platforms, take a look [here](https://www.qemu.org/download).
//!
//! Then, add `ralte32` as a development dependency.
//!
//! ```bash
//! cargo add --dev ralte32
//! ```
//!
//! Lastly, create and/or add a short section to your `.cargo/config.toml`.
//!
//! ```toml
//! # ...
//!
//! [target.riscv32imac-unknown-none-elf]
//! rustflags = ['-Ctarget-feature=+crt-static']
//! runner = "qemu-riscv32 -cpu rv32"
//!
//! # NOTE: If you want to enable additional target features, add them here.
//! #
//! # Example to enable the `zk` feature:
//! # rustflags = ['-Ctarget-feature=+crt-static,+zk']
//! # runner = "qemu-riscv32 -cpu rv32,zk=true"
//! ```
//!
//! Then, to implement some tests, you add an example in `examples/`.
//!
//! ```rust,no_run
//! // examples/test-rv32.rs
//! #![no_std]
//! #![no_main]
//!
//! use ralte32::define_tests;
//!
//! fn test_multiplication() {
//!     assert_eq!(6 * 7, 42);
//! }
//!
//! fn test_remainder() {
//!     assert_eq!(7 % 6, 1);
//! }
//!
//! define_tests!{
//!     test_multiplication,
//!     test_remainder,
//! }
//! ```
//!
//! This can then be ran with:
//!
//! ```bash
//! cargo run --example test-rv32 --target riscv32imac-unknown-none-elf
//! ```
//!
//! This will give:
//!
//! ```text
//! Running tests...
//!
//! Running "test_multiplication"... SUCCESSFUL
//! Running "test_remainder"... SUCCESSFUL
//! ```
//!
//! ## Limitations
//!
//! There are several known limitations.
//!
//! 1. First test or assert to fail, stops the test environment.
//! 2. This only tests user-level code. Access to supervisor, machine or hypervisor instructions
//!    and CSRs is not possible.
//! 3. Very limited support for printing.
//!
//! ## License
//!
//! This project is dual licensed under [MIT](./LICENSE-MIT) and [APACHE-2.0](./LICENSE-APACHE)
//! licenses.
//!
//! [QEMU]: https://www.qemu.org/
//! [RISC-V]: https://en.wikipedia.org/wiki/RISC-V
//! [Rust riscv32 intrinsics]: https://doc.rust-lang.org/nightly/core/arch/riscv32
#![cfg_attr(not(any(not(target_arch = "riscv32"), doc)), no_std)]

#[macro_export]
/// Assert whether an condition is true similar to [`core::assert`].
///
/// This macro has better formatting within the context of this crate.
macro_rules! assert {
    ($condition:expr$(, $txt:literal)?) => {{
        if ! { $condition } {
            $crate::eprint![
                "\n",
                file!(), ":", line!(), ": Assertion failed \"", stringify!($condition), "\"\n",
                $(
                    $txt,
                    "\n",
                )?
            ];
            $crate::syscall::exit(1);
        }
    }};
}

#[macro_export]
/// Assert whether two items are equal similar to [`core::assert_eq`].
///
/// This macro has better formatting within the context of this crate.
macro_rules! assert_eq {
    ($lhs:expr, $rhs:expr$(, $txt:literal)?) => {{
        if ! { $lhs == $rhs } {
            $crate::eprint![
                "\n",
                file!(), ":", line!(), ": Assertion failed. \"", stringify!($lhs), "\" is not equal to \"", stringify!($rhs), "\"\n",
                $(
                    $txt,
                    "\n",
                )?
            ];
            $crate::syscall::exit(1);
        }
    }};
}

#[macro_export]
/// Assert whether two items are not equal similar to [`core::assert_ne`].
///
/// This macro has better formatting within the context of this crate.
macro_rules! assert_ne {
    ($lhs:expr, $rhs:expr$(, $txt:literal)?) => {{
        if ! { $lhs != $rhs } {
            $crate::eprint![
                "\n",
                file!(), ":", line!(), ": Assertion failed. \"", stringify!($lhs), "\" is equal to \"", stringify!($rhs), "\"\n",
                $(
                    $txt,
                    "\n",
                )?
            ];
            $crate::syscall::exit(1);
        }
    }};
}

#[macro_export]
/// Print several items to the standard output.
///
/// This is similar to [`std::print`], but is used without formatting. Instead, all items are
/// specified sequentially.
macro_rules! print {
    ($($item:expr),* $(,)?) => {{
        $(
            $crate::Rv32Write::write(&$item, $crate::buffered_writer::write_stdout);
        )*
        $crate::buffered_writer::flush_stdout();
    }};
}

#[macro_export]
/// Print several items and a newline to the standard output.
///
/// This is similar to [`std::println`], but is used without formatting. Instead, all items are
/// specified sequentially.
macro_rules! println {
    ($($item:expr),* $(,)?) => {{
        $(
            $crate::Rv32Write::write(&$item, $crate::buffered_writer::write_stdout);
        )*
        $crate::Rv32Write::write(&"\n", $crate::buffered_writer::write_stdout);
        $crate::buffered_writer::flush_stdout();
    }};
}

#[macro_export]
/// Print several items to the standard error.
///
/// This is similar to [`std::eprint`], but is used without formatting. Instead, all items are
/// specified sequentially.
macro_rules! eprint {
    ($($item:expr),* $(,)?) => {{
        $(
            $crate::Rv32Write::write(&$item, $crate::buffered_writer::write_stderr);
        )*
        $crate::buffered_writer::flush_stderr();
    }};
}

#[macro_export]
/// Print several items and a newline to the standard error.
///
/// This is similar to [`std::eprintln`], but is used without formatting. Instead, all items are
/// specified sequentially.
macro_rules! eprintln {
    ($($item:expr),* $(,)?) => {{
        $(
            $crate::Rv32Write::write(&$item, $crate::buffered_writer::write_stderr);
        )*
        $crate::Rv32Write::write(&"\n", $crate::buffered_writer::write_stderr);
        $crate::buffered_writer::flush_stderr();
    }};
}

#[macro_export]
/// Define a set of test functions to run.
///
/// This is the main entry into this crate.
macro_rules! define_tests {
    ($($test_fn:ident),* $(,)?) => {
        #[cfg(target_arch = "riscv32")]
        #[panic_handler]
        fn panic(info: &core::panic::PanicInfo) -> ! {
            $crate::eprintln!();

            if let Some(loc) = info.location() {
                $crate::eprint!(
                    loc.file(),
                    ":",
                    loc.line(),
                    ":",
                    loc.column(),
                    " "
                );
            }

            $crate::eprint!("Code panicked");

            if let Some(message) = info.payload().downcast_ref::<&str>() {
                $crate::eprint!(": ", *message);
            }

            $crate::eprintln!();

            $crate::syscall::exit(1)
        }

        // Linux links against the `_start` function specifically.
        #[cfg(target_arch = "riscv32")]
        #[no_mangle]
        pub extern "C" fn _start() -> ! {
            $crate::println!("Running tests...\n");

            $(
            $crate::print!("Running \"", stringify!($test_fn), "\"...");
            $test_fn();
            $crate::println!("\rRunning \"", stringify!($test_fn), "\"... SUCCESSFUL");
            )*


            $crate::syscall::exit(0)
        }

        #[cfg(not(target_arch = "riscv32"))]
        fn main() {
            return;

            #[allow(unreachable_code)]
            {
                $(
                $test_fn();
                )*
            }
        }
    };
}

/// Linux system calls used by this crate.
pub mod syscall {
    #[inline]
    pub fn write(_file_descriptor: u32, _buf: &[u8]) {
        #[cfg(target_arch = "riscv32")]
        unsafe {
            core::arch::asm!(
                "ecall",
                in ("a7") 64,
                in ("a0") _file_descriptor,
                in ("a1") _buf as *const [u8] as *const u8,
                in ("a2") _buf.len(),
            );
        }

        #[cfg(not(target_arch = "riscv32"))]
        {
            // Failed to write. Not running on RISC-V 32
            std::process::abort();
        }
    }

    #[inline]
    pub fn exit(_status_code: u32) -> ! {
        #[cfg(target_arch = "riscv32")]
        unsafe {
            core::arch::asm!(
                "ecall",
                in ("a7") 93,
                in ("a0") _status_code,
                options (noreturn)
            );
        }

        #[cfg(not(target_arch = "riscv32"))]
        {
            // Failed to exit. Not running on RISC-V 32
            std::process::abort();
        }
    }
}

/// Wrapper type to format a unsigned integer with hexadecimal
pub struct Hex<T>(pub T);
/// Wrapper type to format a unsigned integer with binary
pub struct Binary<T>(pub T);

fn write_stdout(buf: &[u8]) {
    syscall::write(1, buf)
}

fn write_stderr(buf: &[u8]) {
    syscall::write(2, buf)
}

/// Trait to write data to a file descriptor
pub trait Rv32Write {
    /// Convert `self` into a set of UTF-8 bytes which get passed to `writer`.
    fn write(&self, writer: fn(&[u8]));
}

impl Rv32Write for &[u8] {
    fn write(&self, writer: fn(&[u8])) {
        writer(self)
    }
}

impl Rv32Write for char {
    fn write(&self, writer: fn(&[u8])) {
        let mut b = [0; 4];
        self.encode_utf8(&mut b);
        writer(&b[0..self.len_utf8()])
    }
}

impl Rv32Write for &str {
    fn write(&self, writer: fn(&[u8])) {
        writer(self.as_bytes())
    }
}

impl Rv32Write for u128 {
    fn write(&self, writer: fn(&[u8])) {
        let mut num = *self;

        if num == 0 {
            writer(b"0");
            return;
        }

        const MAX_DIGITS: usize = (u128::MAX.ilog10() + 1) as usize;

        let num_digits = u32::from(num % 10 != 0) + num.ilog10();

        let mut buf = [0u8; MAX_DIGITS];

        for i in 0..num_digits {
            buf[(num_digits - i - 1) as usize] = b'0' + (num % 10) as u8;
            num /= 10;
        }

        writer(&buf[0..num_digits as usize]);
    }
}

impl Rv32Write for i128 {
    fn write(&self, writer: fn(&[u8])) {
        let num = *self;

        if num < 0 {
            writer(b"-");
        }

        num.unsigned_abs().write(writer);
    }
}

macro_rules! impl_write {
    ($parent:ty, [$($child:ty),+]) => {
        $(
        impl Rv32Write for $child {
            fn write(&self, writer: fn(&[u8])) {
                <$parent>::from(*self).write(writer)
            }
        }
        )+

    };
}

const LUT: [u8; 16] = [
    b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'A', b'B', b'C', b'D', b'E', b'F',
];

macro_rules! impl_binhex {
    ($($t:ty),+) => {
        $(
        impl Rv32Write for Hex<$t> {
            fn write(&self, writer: fn(&[u8])) {
                let num = self.0;

                for i in 0..<$t>::BITS / 8 {
                    if i % 4 == 0 {
                        b'_'.write(writer);
                    }

                    LUT[((num >> ((<$t>::BITS / 8 - i - 1)*4)) & 0xF) as usize].write(writer);
                }
            }
        }

        impl Rv32Write for Binary<$t> {
            fn write(&self, writer: fn(&[u8])) {
                let num = self.0;

                for i in 0..<$t>::BITS {
                    if i % 4 == 0 {
                        b'_'.write(writer);
                    }

                    let c = if (num >> (<$t>::BITS - i - 1)) & 1 == 1 {
                        b'1'
                    } else {
                        b'0'
                    };

                    c.write(writer);
                }
            }
        }
        )+

    };
}

impl_write! { u128, [ u8, u16, u32, u64 ] }
impl_write! { i128, [ i8, i16, i32, i64 ] }
impl_binhex! { u8, u16, u32, u64, u128 }

#[doc(hidden)]
pub mod buffered_writer {
    const PRINTBUF_CAPACITY: usize = 128;
    static mut PRINTBUF: [u8; PRINTBUF_CAPACITY] = [b'A'; PRINTBUF_CAPACITY];
    static mut PRINTBUF_LEN: usize = 0;

    pub fn write(buf: &[u8], back_writer: fn(&[u8])) {
        let current_len = unsafe { PRINTBUF_LEN };
        if current_len
            .checked_add(buf.len())
            .is_some_and(|value| value < PRINTBUF_CAPACITY)
        {
            for (i, c) in buf.iter().enumerate() {
                unsafe { PRINTBUF[current_len + i] = *c };
            }
            unsafe { PRINTBUF_LEN += buf.len() }
        } else {
            self::flush(back_writer);
            back_writer(buf);
        }
    }

    pub fn flush(back_writer: fn(&[u8])) {
        back_writer(unsafe { &PRINTBUF[0..PRINTBUF_LEN] });
        unsafe { PRINTBUF_LEN = 0 }
    }

    pub fn write_stdout(buf: &[u8]) {
        self::write(buf, super::write_stdout);
    }

    pub fn flush_stdout() {
        self::flush(super::write_stdout);
    }

    pub fn write_stderr(buf: &[u8]) {
        self::write(buf, super::write_stderr);
    }

    pub fn flush_stderr() {
        self::flush(super::write_stderr);
    }
}
