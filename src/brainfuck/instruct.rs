extern crate num_traits;

use num_traits::Signed;
use super::integer::BrainfuckInteger;

#[derive(Debug)]
pub enum Instruct<Int: BrainfuckInteger + Signed> {
    Move(isize),
    Add(Int),
    Set(Int),
    Read,
    Write,
    LoopStart(usize),
    LoopEnd(usize),
    WriteStr(Vec<u8>)
}

impl<Int: BrainfuckInteger + Signed> Clone for Instruct<Int> {
    fn clone(&self) -> Self {
        match *self {
            Instruct::Move(val)         => Instruct::Move(val),
            Instruct::Add(val)          => Instruct::Add(val),
            Instruct::Set(val)          => Instruct::Set(val),
            Instruct::Read              => Instruct::Read,
            Instruct::Write             => Instruct::Write,
            Instruct::LoopStart(val)    => Instruct::LoopStart(val),
            Instruct::LoopEnd(val)      => Instruct::LoopEnd(val),
            Instruct::WriteStr(ref val) => Instruct::WriteStr(val.to_vec()),
        }
    }
}