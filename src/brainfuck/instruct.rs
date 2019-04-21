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
