extern crate num_traits;
use super::super::{Brainfuck, BrainfuckInteger, Instruct};

use std::io::Write;

pub fn optimize<Int: BrainfuckInteger + num_traits::Signed>(code: &Brainfuck<Int>, echo: bool) -> std::io::Result<Brainfuck<Int>> {
    let mut opt_code = Brainfuck::new();
    let mut mem = Vec::<Int>::new();
    let mut ptr = 0usize;
    let mut pc  = 0usize;

    loop {
        if let Some(instr) = code.code.get(pc) {
            match *instr {
                Instruct::Move(off) => {
                    pc += 1;
                    if off == std::isize::MIN || (ptr as isize) < -off {
                        // XXX: what to do when pointer < 0?
                        opt_code.push_move(off);
                        break;
                    }
                    ptr = ((ptr as isize) + off) as usize;
                },

                Instruct::Add(val) => {
                    pc += 1;
                    if ptr >= mem.len() {
                        mem.resize(ptr + 1, Int::zero());
                    }
                    mem[ptr] = mem[ptr].wrapping_add(&val);
                },

                Instruct::Set(val) => {
                    pc += 1;
                    if ptr >= mem.len() {
                        mem.resize(ptr + 1, Int::zero());
                    }
                    mem[ptr] = val;
                },

                Instruct::Read => {
                    pc += 1;
                    opt_code.push_read();
                    break;
                },

                Instruct::Write => {
                    pc += 1;
                    if ptr >= mem.len() {
                        mem.resize(ptr + 1, Int::zero());
                    }
                    let data = vec![mem[ptr].get_least_byte()];
                    if echo {
                        std::io::stdout().write_all(&data)?;
                    }
                    opt_code.push_write_str(data);
                },

                Instruct::WriteStr(ref data) => {
                    pc += 1;
                    if echo {
                        std::io::stdout().write_all(data)?;
                    }
                    opt_code.push_write_str(data.to_vec());
                },

                Instruct::LoopStart(pc_false) => {
                    if ptr >= mem.len() {
                        mem.resize(ptr + 1, Int::zero());
                    }
                    if mem[ptr] == Int::zero() {
                        pc = pc_false;
                    } else {
                        pc += 1;
                    }
                },

                Instruct::LoopEnd(pc_loop_start) => {
                    pc = pc_loop_start;
                }
            }
        } else {
            break;
        }
    }

    if pc < code.code.len() {
        let mut current_ptr = ptr;
        for (target_ptr, val) in mem.iter().enumerate() {
            if *val != Int::zero() {
                if current_ptr != target_ptr {
                    let off = (target_ptr as isize) - (ptr as isize);
                    opt_code.push_move(off);
                    current_ptr = target_ptr;
                }
                opt_code.push_set(*val);
            }
        }

        if current_ptr != ptr {
            let off = (ptr as isize) - (ptr as isize);
            opt_code.push_move(off);
        }

        while let Some(instr) = code.code.get(pc) {
            opt_code.push_ref(instr);
            pc += 1;
        }
    }

    return Ok(opt_code);
}