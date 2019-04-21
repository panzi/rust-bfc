
extern crate num_traits;

use num_traits::{PrimInt, WrappingShl, WrappingAdd};

pub trait BrainfuckInteger: PrimInt + WrappingShl + WrappingAdd + std::fmt::Debug {
    fn c_type() -> &'static str;
    fn get_least_byte(self) -> u8;
    fn from_byte(value: u8) -> Self;
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
        return "uint8_t";
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
        return "int8_t";
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
        return "uint16_t";
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
        return "int16_t";
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
        return "uint32_t";
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
        return "int32_t";
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
        return "uint64_t";
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
        return "int64_t";
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
        return "ssize_t";
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
        return "size_t";
    }
}
