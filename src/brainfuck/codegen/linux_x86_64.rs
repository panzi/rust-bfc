extern crate num_traits;
extern crate regex;

use num_traits::Signed;
use regex::Regex;
use std::fs::File;
use std::io::Write;
use std::collections::HashMap;
use std::ffi::OsStr;
use super::super::{Brainfuck, BrainfuckInteger, Instruct};
use super::generate_c_write_str::generate_c_write_str;
use super::generate_asm_str::generate_asm_str;
use super::generate_c_runtime::generate_c_runtime;

pub fn generate<Int: BrainfuckInteger + Signed>(code: &Brainfuck<Int>, binary_file: &str) -> std::io::Result<Vec<String>> {
    let mut filenames = Vec::new();
    let mut min_move = 0isize;
    let mut max_move = 0isize;
    let mut cur_move = 0isize;
    let mut uses_mem = false;
    let mut last_was_move = false;
    let mut nesting = 1usize;

    for instr in code.iter() {
        match instr {
            Instruct::Move(off) => {
                if last_was_move {
                    cur_move += *off;
                } else {
                    cur_move = *off;
                    last_was_move = true;
                }

                if cur_move > max_move {
                    max_move = cur_move;
                }

                if cur_move < min_move {
                    min_move = cur_move;
                }
            },

            Instruct::Add(_) => {
                uses_mem = true;
                last_was_move = false;
            },

            Instruct::Set(_) => {
                uses_mem = true;
                last_was_move = false;
            },

            Instruct::AddTo(off) => {
                let add_to_move = cur_move + off;
                if add_to_move > max_move {
                    max_move = add_to_move;
                }

                if add_to_move < min_move {
                    min_move = add_to_move;
                }
                uses_mem = true;
                last_was_move = false;
            },

            Instruct::Read => {
                uses_mem = true;
                last_was_move = false;
            },

            Instruct::Write => {
                uses_mem = true;
                last_was_move = false;
            },

            Instruct::LoopStart(_) => {
                uses_mem = true;
                last_was_move = false;
            },

            Instruct::LoopEnd(_) => {
                uses_mem = true;
                last_was_move = false;
            },

            Instruct::WriteStr(_) => {}
        }
    }

    if uses_mem {
        if -min_move > max_move {
            max_move = -min_move;
        }

        let pagesize = ((max_move as usize * std::mem::size_of::<Int>() / 4096) + 1) * 4096;

        let runtime_src_filename = format!("{}-runtime.c", binary_file);
        let mut runtime = File::create(&runtime_src_filename)?;
        filenames.push(runtime_src_filename);

        generate_c_runtime(&mut runtime, Int::c_type(), pagesize)?;

        let mut str_table = HashMap::new();
        let mut loop_stack = Vec::new();
        let mut loop_count = 0usize;

        let bf_src_filename = format!("{}.asm", binary_file);
        let mut asm = File::create(&bf_src_filename)?;
        filenames.push(bf_src_filename);

        for instr in code.iter() {
            if let Instruct::WriteStr(data) = instr {
                if data.len() > 1 && !str_table.contains_key(data) {
                    str_table.insert(data, str_table.len());
                }
            }
        }

        asm.write_all(
br##"        bits 64
        section .data
"##)?;

        for (msg, index) in str_table.iter() {
            let name = format!("msg{}", index);
            generate_asm_str(&mut asm, &name, msg)?;
        }

        write!(asm,
"        section .text
        extern stdout
        extern fwrite
        extern putchar
        extern getchar
        extern fflush
        extern mem
        global bfmain
bfmain:
        push rbp
        mov  rbp, rsp
        push r12
        mov  qword  r12 , [rel mem]
        add  qword  r12 , {:8} ; {}* ptr = (void*)mem + PAGESIZE;
", pagesize, Int::c_type())?;

        let int_size = std::mem::size_of::<Int>() as isize;
        let prefix = match int_size {
            1 => "byte ",
            2 => "word ",
            4 => "dword",
            8 => "qword",
            x => panic!("unsupported cell size: {}", x),
        };
        let reg = match int_size {
            1 => "al",
            2 => "ax",
            4 => "eax",
            8 => "rax",
            x => panic!("unsupported cell size: {}", x),
        };
        nesting = 0;
        let mut pc = 0;
        loop {
            if let Some(instr) = code.get(pc) {
                match *instr {
                    Instruct::Move(off) => {
                        if int_size == 1 && off == 1 {
                            write!(asm, "        inc  qword  r12            ; {:nesting$}ptr ++;\n", "", nesting = nesting)?;
                        } else if int_size == 1 && off == -1 {
                            write!(asm, "        dec  qword  r12            ; {:nesting$}ptr --;\n", "", nesting = nesting)?;
                        } else if off > 0 {
                            let val = off * int_size;
                            write!(asm, "        add  qword  r12 , {:8} ; {:nesting$}ptr  += {};\n", val, "", off, nesting = nesting)?;
                        } else if off != 0 {
                            let val = -off * int_size;
                            write!(asm, "        sub  qword  r12 , {:8} ; {:nesting$}ptr  -= {};\n", val, "", -off, nesting = nesting)?;
                        }
                        pc += 1;
                    },

                    Instruct::Add(val) => {
                        let v = val.as_i64();
                        if v == 1 {
                            write!(asm, "        inc  {} [r12]           ; {:nesting$}*ptr += 1;\n", prefix, "", nesting = nesting)?;
                        } else if v == -1 {
                            write!(asm, "        dec  {} [r12]           ; {:nesting$}*ptr -= 1;\n", prefix, "", nesting = nesting)?;
                        } else if v > 0 {
                            write!(asm, "        add  {} [r12], {:8} ; {:nesting$}*ptr += {};\n", prefix, v, "", v, nesting = nesting)?;
                        } else if v != 0 {
                            write!(asm, "        sub  {} [r12], {:8} ; {:nesting$}*ptr -= {};\n", prefix, -v, "", -v, nesting = nesting)?;
                        }
                        pc += 1;
                    },

                    Instruct::Set(val) => {
                        write!(asm, "        mov  {} [r12], {:8} ; {:nesting$}*ptr  = {};\n", prefix, val.as_i64(), "", val.as_i64(), nesting = nesting)?;
                        pc += 1;
                    },

                    Instruct::AddTo(_) => {
                        loop_count += 1;

                        if let Some(val) = code.find_set_before(pc) {
                            if val != Int::zero() {
                                while let Some(Instruct::AddTo(off)) = code.get(pc) {
                                    let dest = if *off > 0 {
                                        format!("[r12+{}]", *off * int_size)
                                    } else {
                                        format!("[r12-{}]", -*off * int_size)
                                    };
                                    let padding = if dest.len() >= 14 { 0 } else { 14 - dest.len() };
                                    write!(asm, "        add  {} {}, {:padding$}; {:nesting$}ptr[{}] += *ptr;\n",
                                        prefix, dest, val.as_i64(), "", off, nesting = nesting, padding = padding)?;
                                    pc += 1;
                                }
                            } else {
                                while let Some(Instruct::AddTo(_)) = code.get(pc) {
                                    pc += 1;
                                }
                            }
                        } else {
                            write!(asm, "        mov         {:3} , [r12]\n", reg)?;
                            let mut current_off = 0isize;
                            while let Some(Instruct::AddTo(off)) = code.get(pc) {
                                if current_off != *off {
                                    generate_move(&mut asm, *off - current_off)?;
                                }
                                write!(asm, "        add  {} [r12], {:9}; {:nesting$}ptr[{}] += *ptr;\n",
                                    prefix, reg, "", *off, nesting = nesting)?;
                                current_off = *off;
                                pc += 1;
                            }
                            let mut target_off = -current_off;

                            // if there is a move now, we can do this both at once
                            if let Some(Instruct::Move(off)) = code.get(pc) {
                                target_off += *off;
                                pc += 1;
                                // TODO: this eats the comment of this move
                            }
                            generate_move(&mut asm, target_off)?;
                        }
                    },

                    Instruct::Read => {
                        write!(asm, "        mov  rdi, [rel stdout]\n")?;
                        write!(asm, "        call fflush                ; {:nesting$}fflush(stdout);\n", "", nesting = nesting)?;

                        write!(asm, "        call getchar\n")?;
                        write!(asm, "        mov  {} [r12], {:7}      ; {:nesting$}*ptr = getchar();\n", prefix, reg, "", nesting = nesting)?;
                        pc += 1;
                    },

                    Instruct::Write => {
                        write!(asm, "        mov  edi,  [r12]\n")?;
                        write!(asm, "        call putchar               ; {:nesting$}putchar(*ptr)\n", "", nesting = nesting)?;
                        pc += 1;
                    },

                    Instruct::LoopStart(pc_loop_end) => {
                        loop_count += 1;

                        if let Some(val) = code.find_set_before(pc) {
                            if val == Int::zero() {
                                pc = pc_loop_end;
                            } else {
                                loop_stack.push(loop_count);
                                write!(asm, "start{}:                           ; {:nesting$}do {{\n", loop_count, "", nesting = nesting)?;
                                nesting += 4;
                                pc += 1;
                            }
                        } else {
                            loop_stack.push(loop_count);
                            let stmt = if let Some(Instruct::Set(val2)) = code.get(pc_loop_end - 2) {
                                if *val2 == Int::zero() { "if" } else { "while" }
                            } else { "while" };

                            write!(asm, "        cmp  {} [r12],        0 ; {:nesting$}{} (*ptr) {{\n", prefix, "", stmt, nesting = nesting)?;
                            write!(asm, "        je   end{}\n", loop_count)?;
                            write!(asm, "start{}:\n", loop_count)?;
                            nesting += 4;
                            pc += 1;
                        }
                    },

                    Instruct::LoopEnd(pc_start) => {
                        nesting -= 4;
                        let loop_id = loop_stack.pop().unwrap();
                        let stmt = if code.find_set_before(pc_start).is_some() {
                            "} while (*ptr);"
                        } else { "}" };

                        if let Some(val) = code.find_set_before(pc) {
                            if val == Int::zero() {
                                write!(asm, "                                   ; {:nesting$}{}\n", "", stmt, nesting = nesting)?;
                            } else {
                                // This would be an infinite loop, right?
                                write!(asm, "        jmp  {:7} ; {:nesting$}{}\n", format!("start{}", loop_id), "", stmt, nesting = nesting)?;
                            }
                        } else {
                            write!(asm, "        cmp  {} [r12],        0 ; {:nesting$}{}\n", prefix, "", stmt, nesting = nesting)?;
                            write!(asm, "        jne  start{}\n", loop_id)?;
                        }

                        write!(asm, "end{}:\n", loop_id)?;
                        pc += 1;
                    },

                    Instruct::WriteStr(ref data) => {
                        if data.len() == 1 {
                            write!(asm, "        mov  edi, {}\n", data[0])?;
                            write!(asm, "        call putchar               ; {:nesting$}putchar({})\n", "", data[0], nesting = nesting)?;
                        } else if data.len() > 0 {
                            let msg_id = str_table.get(data).unwrap();

                            write!(asm, "        mov  rcx, [rel stdout]\n")?;
                            write!(asm, "        mov  edx, 1\n")?;
                            write!(asm, "        mov  esi, {}\n", data.len())?;
                            write!(asm, "        mov  edi, msg{}\n", msg_id)?;
                            write!(asm, "        call fwrite                ; {:nesting$}fwrite(msg{}, {}, 1, stdout);\n", "", msg_id, data.len(), nesting = nesting)?;
                        }
                        pc += 1;
                    },
                }
            } else {
                break;
            }
        }

        asm.write_all(
b"        pop  r12
        mov  rsp, rbp
        pop  rbp
        ret
")?;

    } else {
        let c_filename = format!("{}.c", binary_file);
        let mut out = File::create(&c_filename)?;
        let mut need_flush = false;
        write!(out, r##"#include <stdio.h>

int main() {{
"##)?;

        for instr in code.iter() {
            if let Instruct::WriteStr(data) = instr {
                if data.len() > 0 {
                    generate_c_write_str(&mut out, data, nesting)?;
                    need_flush = data[data.len() - 1] != b'\n';
                }
            }
        }

        if need_flush {
            out.write_all(b"    fflush(stdout);\n")?;
        }

        out.write_all(b"\n    return 0;\n}\n")?;

        filenames.push(c_filename);
    }

    return Ok(filenames);
}

fn generate_move(asm: &mut Write, off: isize) -> std::io::Result<()> {
    if off == 1 {
        write!(asm, "        inc  qword r12\n")
    } else if off == -1 {
        write!(asm, "        dec  qword r12\n")
    } else if off < 0 {
        write!(asm, "        sub  qword r12, {}\n", -off)
    } else if off > 0 {
        write!(asm, "        add  qword r12, {}\n", off)
    } else {
        Ok(())
    }
}

pub fn compile_c(source_file: &str, object_file: &str, debug: bool, optlevel: u32) -> std::io::Result<()> {
    let cc = if let Ok(cc) = std::env::var("CC") {
        cc
    } else {
        "gcc".to_string()
    };
    let mut cmd = std::process::Command::new(&cc);
    if let Ok(cflags) = std::env::var("CFLAGS") {
        for flag in cflags.split_whitespace() {
            cmd.arg(flag);
        }
    }
    if debug {
        cmd.arg("-g")
           .arg("-DDEBUG");
    }
    let status = cmd
        .arg(format!("-O{}", optlevel))
        .arg("-Wall")
        .arg("-Wextra")
        .arg("-std=gnu11")
        .arg("-c")
        .arg("-o")
        .arg(object_file)
        .arg(source_file)
        .status()?;

    if !status.success() {
        let message = if let Some(code) = status.code() {
            format!("{} exited with status {}", cc, code)
        } else {
            format!("{} terminated by signal", cc)
        };
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            message));
    }

    return Ok(());
}

pub fn assemble(source_file: &str, object_file: &str, debug: bool, optlevel: u32) -> std::io::Result<()> {
    let asm = if let Ok(asm) = std::env::var("ASM") {
        asm
    } else {
        "nasm".to_string()
    };
    let mut cmd = std::process::Command::new(&asm);
    if let Ok(asmflags) = std::env::var("ASMFLAGS") {
        for flag in asmflags.split_whitespace() {
            cmd.arg(flag);
        }
    }
    if debug {
        cmd.arg("-g")
           .arg("-F")
           .arg("dwarf");
    }
    let status = cmd
        .arg("-f")
        .arg("elf64")
        .arg(format!("-O{}", optlevel))
        .arg("-o")
        .arg(object_file)
        .arg(source_file)
        .status()?;

    if !status.success() {
        let message = if let Some(code) = status.code() {
            format!("{} exited with status {}", asm, code)
        } else {
            format!("{} terminated by signal", asm)
        };
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            message));
    }

    return Ok(());
}

pub fn link<I, S>(obj_files: I, binary_file: &str, debug: bool, optlevel: u32) -> std::io::Result<()>
        where I: IntoIterator<Item=S>, S: AsRef<OsStr> {
    let ld = if let Ok(ld) = std::env::var("LD") {
        ld
    } else if let Ok(cc) = std::env::var("CC") {
        cc
    } else {
        "gcc".to_string()
    };
    let mut cmd = std::process::Command::new(&ld);
    if let Ok(ldflags) = std::env::var("LDFLAGS") {
        for flag in ldflags.split_whitespace() {
            cmd.arg(flag);
        }
    }
    if debug {
        cmd.arg("-g");
    }
    let status = cmd
        .arg(format!("-O{}", optlevel))
        .arg("-o")
        .arg(binary_file)
        .args(obj_files)
        .status()?;

    if !status.success() {
        let message = if let Some(code) = status.code() {
            format!("{} exited with status {}", ld, code)
        } else {
            format!("{} terminated by signal", ld)
        };
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            message));
    }

    return Ok(());
}

pub fn compile<Int: BrainfuckInteger + Signed>(code: &Brainfuck<Int>, binary_file: &str, debug: bool, optlevel: u32, keep_source: bool) -> std::io::Result<()> {
    let filenames = generate(code, &binary_file)?;
    let mut obj_files = Vec::new();
    let c_re   = Regex::new(r"\.c$").unwrap();
    let asm_re = Regex::new(r"\.asm$").unwrap();

    for filename in &filenames {
        if filename.ends_with(".c") {
            let obj_file = format!("{}.o", c_re.replace(&filename, ""));
            compile_c(&filename, &obj_file, debug, optlevel)?;
            obj_files.push(obj_file);
        } else {
            let obj_file = format!("{}.o", asm_re.replace(&filename, ""));
            assemble(&filename, &obj_file, debug, optlevel)?;
            obj_files.push(obj_file);
        }
    }

    link(&obj_files, &binary_file, debug, optlevel)?;
    
    if !keep_source {
        for filename in &filenames {
            std::fs::remove_file(filename)?;
        }
    }

    for filename in &obj_files {
        std::fs::remove_file(filename)?;
    }
    return Ok(());
}