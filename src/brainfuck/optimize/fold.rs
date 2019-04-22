extern crate num_traits;
use super::super::{Brainfuck, BrainfuckInteger, Instruct};

pub fn optimize<Int: BrainfuckInteger + num_traits::Signed>(code: &Brainfuck<Int>) -> Brainfuck<Int> {
    let mut opt_code = Brainfuck::new();
    let mut index = 0usize;

    loop {
        if let Some(instr) = code.code.get(index) {
            index += 1;
            match *instr {
                Instruct::Move(val1) => {
                    let mut val = val1;
                    while let Some(Instruct::Move(val2)) = code.code.get(index) {
                        index += 1;
                        val += *val2;
                    }
                    opt_code.push_move(val);
                },
                Instruct::Add(val1) => {
                    let mut val = val1;
                    while let Some(Instruct::Add(val2)) = code.code.get(index) {
                        index += 1;
                        val = val + *val2;
                    }
                    opt_code.push_add(val);
                },
                Instruct::Set(val1) => {
                    let mut val = val1;
                    while let Some(Instruct::Set(val2)) = code.code.get(index) {
                        index += 1;
                        val = *val2;
                    }
                    opt_code.push_set(val);
                },
                _ => opt_code.push(instr)
            }
        } else {
            break;
        }
    }

    return opt_code;
}