extern crate num_traits;
use super::super::{Brainfuck, BrainfuckInteger, Instruct};
use std::collections::HashSet;

pub fn optimize<Int: BrainfuckInteger + num_traits::Signed>(code: &Brainfuck<Int>) -> Brainfuck<Int> {
    let mut opt_code = Brainfuck::new();
    let mut index = 0usize;

    // a loop with not sub loops, no IO
    // all moves add up to 0
    // the loop variable is decreased by one
    // other cells are increased by one

    loop {
        match code.get(index) {
            Some(Instruct::LoopStart(_)) => {
                let mut offsets = HashSet::new();
                let mut offset = 0isize;
                let mut end_index = index + 1;
                let mut decreased = false;
                let matched = loop {
                    match code.get(end_index) {
                        Some(Instruct::Move(off)) => {
                            offset += *off;
                            end_index += 1;
                        },
                        Some(Instruct::Add(val)) => {
                            match val.as_i64() {
                                -1 if offset == 0 && !decreased => {
                                    decreased = true;
                                },
                                1 if offset != 0 && !offsets.contains(&offset) => {
                                    offsets.insert(offset);
                                },
                                _ => {
                                    break false;
                                }
                            }
                            end_index += 1;
                        },
                        Some(Instruct::LoopEnd(_)) if offset == 0 => {
                            break decreased;
                        },
                        _ => {
                            break false;
                        }
                    }
                };

                if matched {
                    let mut sorted_offsets = Vec::with_capacity(offsets.len());
                    for offset in offsets {
                        sorted_offsets.push(offset);
                    }
                    sorted_offsets.sort_unstable();
                    for offset in sorted_offsets {
                        opt_code.push_add_to(offset);
                    }
                    opt_code.push_set(Int::zero());
                    index = end_index + 1;
                } else {
                    opt_code.push_loop_start();
                    index += 1;
                }
            },
            Some(instr) => {
                opt_code.push(instr);
                index += 1;
            },
            None => break
        }
    }

    return opt_code;
}