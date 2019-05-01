extern crate num_traits;
use super::super::{Brainfuck, BrainfuckInteger, Instruct};

fn unchanged_ptr_loop_end<Int>(code: &Brainfuck<Int>, mut current_off: isize, target_off: isize, mut index: usize) -> Option<usize>
    where Int: BrainfuckInteger + num_traits::Signed {
    loop {
        if let Some(instr) = code.get(index) {
            index += 1;
            match *instr {
                Instruct::Set(_) | Instruct::Read => {
                    // might not happen depending on loop condition
                },
                Instruct::AddTo(_) | Instruct::SubFrom(_) | Instruct::Write => {
                    // might not happen depending on loop condition
                },
                Instruct::Add(_) | Instruct::WriteStr(_) => {},
                Instruct::Move(off) => {
                    current_off += off;
                },
                Instruct::LoopStart(_) => {
                    if let Some(end_index) = unchanged_ptr_loop_end(code, current_off, target_off, index) {
                        index = end_index;
                    } else {
                        return None;
                    }
                },
                Instruct::LoopEnd(_) => {
                    if current_off == target_off {
                        return Some(index);
                    }
                    return None;
                }
            }
        } else {
            return None;
        }
    }
}

fn has_set_after<Int>(code: &Brainfuck<Int>, target_off: isize, mut index: usize) -> bool
    where Int: BrainfuckInteger + num_traits::Signed {
    let mut current_off = 0;
    loop {
        if let Some(instr) = code.get(index) {
            index += 1;
            match *instr {
                Instruct::Set(_) | Instruct::Read => {
                    if current_off == target_off {
                        return true;
                    }
                },
                Instruct::AddTo(_) | Instruct::SubFrom(_) | Instruct::Write => {
                    if current_off == target_off {
                        return false;
                    }
                },
                Instruct::Add(_) | Instruct::WriteStr(_) => {},
                Instruct::Move(off) => {
                    current_off += off;
                },
                Instruct::LoopStart(_) => {
                    if let Some(end_index) = unchanged_ptr_loop_end(code, current_off, target_off, index) {
                        index = end_index;
                    } else {
                        return false;
                    }
                },
                Instruct::LoopEnd(_) => {
                    return false;
                }
            }
        } else {
            break;
        }
    }
    return false;
}

pub fn optimize<Int: BrainfuckInteger + num_traits::Signed>(code: &Brainfuck<Int>) -> Brainfuck<Int> {
    let mut opt_code = Brainfuck::new();
    let mut index = 0usize;

    loop {
        // this partially re-scans the same part again and again
        // could be done more efficiently
        if let Some(instr) = code.get(index) {
            index += 1;
            match *instr {
                Instruct::Set(_) | Instruct::Add(_) => {
                    if !has_set_after(code, 0, index) {
                        opt_code.push(instr);
                    }
                },
                Instruct::AddTo(off) | Instruct::SubFrom(off) => {
                    if !has_set_after(code, off, index) {
                        opt_code.push(instr);
                    }
                },
                _ => opt_code.push(instr),
            }
        } else {
            break;
        }
    }

    return opt_code;
}