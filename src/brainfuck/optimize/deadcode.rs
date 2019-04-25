extern crate num_traits;
use super::super::{Brainfuck, BrainfuckInteger, Instruct};

pub fn optimize<Int: BrainfuckInteger + num_traits::Signed>(code: &Brainfuck<Int>) -> Brainfuck<Int> {
    let mut opt_code = Brainfuck::new();
    let mut index = 0usize;

    loop {
        match (code.get(index), code.get(index + 1)) {
            (Some(Instruct::Set(val)), Some(Instruct::LoopStart(end_index))) => {
                index += 2;

                if *val == Int::zero() {
                    index = *end_index;
                } else {
                    opt_code.push_set(*val);
                    opt_code.push_loop_start();
                }
            },
            (Some(instr), _) => {
                opt_code.push(instr);
                index += 1;
            },
            (None, _) => break
        }
    }

    return opt_code;
}