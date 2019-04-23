extern crate num_traits;
use super::super::{Brainfuck, BrainfuckInteger, Instruct};

fn optimize_write_str<Int: BrainfuckInteger + num_traits::Signed>(code: &Brainfuck<Int>, mut index: usize, mut last_val: Int, data: &mut Vec<u8>) -> (usize, Int) {
    loop {
        if let (Some(Instruct::Set(val)), Some(Instruct::Write)) = (code.code.get(index), code.code.get(index + 1)) {
            index += 2;
            last_val = *val;
            data.push(val.get_least_byte());
        } else if let Some(Instruct::Write) = code.code.get(index) {
            index += 1;
            data.push(last_val.get_least_byte());
        } else if let Some(Instruct::WriteStr(data2)) = code.code.get(index) {
            index += 1;
            data.extend(data2);
        } else {
            break;
        }
    }
    return (index, last_val);
}

pub fn optimize<Int: BrainfuckInteger + num_traits::Signed>(code: &Brainfuck<Int>) -> Brainfuck<Int> {
    let mut opt_code = Brainfuck::new();
    let mut index = 0usize;

    loop {
        match (code.code.get(index), code.code.get(index + 1)) {
            (Some(Instruct::Set(val)), Some(Instruct::Write)) => {
                index += 2;
                let mut data = vec![val.get_least_byte()];
                let (new_index, last_val) = optimize_write_str(code, index, *val, &mut data);
                index = new_index;
                // preserve last value, it might be used!
                opt_code.push_set(last_val);
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
                    let last_val = Int::from_byte(data[data.len() - 1]);
                    let (new_index, _) = optimize_write_str(code, index, last_val, &mut data);
                    index = new_index;
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