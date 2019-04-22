extern crate num_traits;
use super::super::{Brainfuck, BrainfuckInteger, Instruct};

fn optimize_write_str<Int: BrainfuckInteger + num_traits::Signed>(code: &Brainfuck<Int>, mut index: usize, data: &mut Vec<u8>) -> usize {
    let mut last_val = data[data.len() - 1];
    loop {
        if let (Some(Instruct::Set(val)), Some(Instruct::Write)) = (code.code.get(index), code.code.get(index + 1)) {
            index += 2;
            last_val = val.get_least_byte();
            data.push(last_val);
        } else if let Some(Instruct::Write) = code.code.get(index) {
            index += 1;
            data.push(last_val);
        } else if let Some(Instruct::WriteStr(data2)) = code.code.get(index) {
            index += 1;
            data.extend(data2);
        } else {
            break;
        }
    }
    return index;
}

pub fn optimize<Int: BrainfuckInteger + num_traits::Signed>(code: &Brainfuck<Int>) -> Brainfuck<Int> {
    let mut opt_code = Brainfuck::new();
    let mut index = 0usize;

    loop {
        match (code.code.get(index), code.code.get(index + 1)) {
            (Some(Instruct::Set(val)), Some(Instruct::Write)) => {
                index += 2;
                let mut data = vec![val.get_least_byte()];
                index = optimize_write_str(code, index, &mut data);
                opt_code.push_write_str(data);
                continue;
            },
            _ => {}
        }

        if let Some(instr) = code.code.get(index) {
            index += 1;
            if let Instruct::WriteStr(data) = instr {
                if data.len() > 0 {
                    let mut data = data.to_vec();
                    index = optimize_write_str(code, index, &mut data);
                    opt_code.push_write_str(data);
                }
            } else {
                opt_code.push(instr);
            }
        } else {
            break;
        }
    }

    return opt_code;
}