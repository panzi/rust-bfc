extern crate num_traits;
use super::super::{Brainfuck, BrainfuckInteger, Instruct};

pub fn optimize<Int: BrainfuckInteger + num_traits::Signed>(code: &Brainfuck<Int>) -> Brainfuck<Int> {
    let mut opt_code = Brainfuck::new();
    let mut index = 0usize;

    loop {
        match (code.code.get(index), code.code.get(index + 1), code.code.get(index + 2), code.code.get(index + 3)) {
            (Some(Instruct::LoopStart(_)), Some(Instruct::Add(_)), Some(Instruct::LoopEnd(_)), Some(Instruct::Add(val))) => {
                index += 4;
                opt_code.push_set(*val);
                continue;
            },
            _ => {}
        }

        if let (Some(Instruct::LoopStart(_)), Some(Instruct::Add(_)), Some(Instruct::LoopEnd(_))) =
                (code.code.get(index), code.code.get(index + 1), code.code.get(index + 2)) {
            index += 3;
            opt_code.push_set(Int::zero());
            continue;
        }

        if let Some(instr) = code.code.get(index) {
            index += 1;
            opt_code.push(instr);
        } else {
            break;
        }
    }

    return opt_code;
}