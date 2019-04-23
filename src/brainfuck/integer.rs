
extern crate num_traits;

use num_traits::{PrimInt, WrappingShl, WrappingAdd};

pub trait BrainfuckInteger: PrimInt + WrappingShl + WrappingAdd + std::fmt::Debug {
    fn c_type() -> &'static str;
    fn get_least_byte(self) -> u8;
    fn from_byte(value: u8) -> Self;
    fn wrapping_usize(self) -> usize;
    fn i64(self) -> i64;
    fn nasm_prefix() -> &'static str;
}

impl BrainfuckInteger for u8 {
    #[inline]
    fn get_least_byte(self) -> u8 {
        self
    }

    #[inline]
    fn from_byte(value: u8) -> Self {
        value
    }

    fn c_type() -> &'static str {
        "uint8_t"
    }

    fn wrapping_usize(self) -> usize {
        self as usize
    }

    fn i64(self) -> i64 {
        self as i64
    }

    fn nasm_prefix() -> &'static str {
        "byte"
    }
}

impl BrainfuckInteger for i8 {
    #[inline]
    fn get_least_byte(self) -> u8 {
        self as u8
    }

    #[inline]
    fn from_byte(value: u8) -> Self {
        value as i8
    }

    fn c_type() -> &'static str {
        "int8_t"
    }

    fn wrapping_usize(self) -> usize {
        self as usize
    }

    fn i64(self) -> i64 {
        self as i64
    }

    fn nasm_prefix() -> &'static str {
        "byte"
    }
}

impl BrainfuckInteger for u16 {
    #[inline]
    fn get_least_byte(self) -> u8 {
        (self & 0xFF) as u8
    }

    #[inline]
    fn from_byte(value: u8) -> Self {
        value as u16
    }

    fn c_type() -> &'static str {
        "uint16_t"
    }

    fn wrapping_usize(self) -> usize {
        self as usize
    }

    fn i64(self) -> i64 {
        self as i64
    }

    fn nasm_prefix() -> &'static str {
        "word"
    }
}

impl BrainfuckInteger for i16 {
    #[inline]
    fn get_least_byte(self) -> u8 {
        (self & 0xFF) as u8
    }

    #[inline]
    fn from_byte(value: u8) -> Self {
        value as i16
    }

    fn c_type() -> &'static str {
        "int16_t"
    }

    fn wrapping_usize(self) -> usize {
        self as usize
    }

    fn i64(self) -> i64 {
        self as i64
    }

    fn nasm_prefix() -> &'static str {
        "word"
    }
}

impl BrainfuckInteger for u32 {
    #[inline]
    fn get_least_byte(self) -> u8 {
        (self & 0xFF) as u8
    }

    #[inline]
    fn from_byte(value: u8) -> Self {
        value as u32
    }

    fn c_type() -> &'static str {
        "uint32_t"
    }

    fn wrapping_usize(self) -> usize {
        self as usize
    }

    fn i64(self) -> i64 {
        self as i64
    }

    fn nasm_prefix() -> &'static str {
        "dword"
    }
}

impl BrainfuckInteger for i32 {
    #[inline]
    fn get_least_byte(self) -> u8 {
        (self & 0xFF) as u8
    }

    #[inline]
    fn from_byte(value: u8) -> Self {
        value as i32
    }

    fn c_type() -> &'static str {
        "int32_t"
    }

    fn wrapping_usize(self) -> usize {
        self as usize
    }

    fn i64(self) -> i64 {
        self as i64
    }

    fn nasm_prefix() -> &'static str {
        "dword"
    }
}

impl BrainfuckInteger for u64 {
    #[inline]
    fn get_least_byte(self) -> u8 {
        (self & 0xFF) as u8
    }

    #[inline]
    fn from_byte(value: u8) -> Self {
        value as u64
    }

    fn c_type() -> &'static str {
        "uint64_t"
    }

    fn wrapping_usize(self) -> usize {
        self as usize
    }

    fn i64(self) -> i64 {
        self as i64
    }

    fn nasm_prefix() -> &'static str {
        "qword"
    }
}

impl BrainfuckInteger for i64 {
    #[inline]
    fn get_least_byte(self) -> u8 {
        (self & 0xFF) as u8
    }

    #[inline]
    fn from_byte(value: u8) -> Self {
        value as i64
    }

    fn c_type() -> &'static str {
        "int64_t"
    }

    fn wrapping_usize(self) -> usize {
        self as usize
    }

    fn i64(self) -> i64 {
        self as i64
    }

    fn nasm_prefix() -> &'static str {
        "qword"
    }
}

impl BrainfuckInteger for isize {
    #[inline]
    fn get_least_byte(self) -> u8 {
        (self & 0xFF) as u8
    }

    #[inline]
    fn from_byte(value: u8) -> Self {
        value as isize
    }

    fn c_type() -> &'static str {
        "ssize_t"
    }

    fn wrapping_usize(self) -> usize {
        self as usize
    }

    fn i64(self) -> i64 {
        self as i64
    }

    fn nasm_prefix() -> &'static str {
        match std::mem::size_of::<isize>() {
            1 => "byte",
            2 => "word",
            4 => "dword",
            8 => "qword",
            x => panic!("unsupported size: {}", x),
        }
    }
}

impl BrainfuckInteger for usize {
    #[inline]
    fn get_least_byte(self) -> u8 {
        (self & 0xFF) as u8
    }

    #[inline]
    fn from_byte(value: u8) -> Self {
        value as usize
    }

    fn c_type() -> &'static str {
        "size_t"
    }

    fn i64(self) -> i64 {
        self as i64
    }

    fn wrapping_usize(self) -> usize {
        self as usize
    }

    fn nasm_prefix() -> &'static str {
        match std::mem::size_of::<isize>() {
            1 => "byte",
            2 => "word",
            4 => "dword",
            8 => "qword",
            x => panic!("unsupported size: {}", x),
        }
    }
}
