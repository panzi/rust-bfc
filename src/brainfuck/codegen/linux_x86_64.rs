extern crate num_traits;
extern crate regex;

use num_traits::Signed;
use regex::Regex;
use std::fs::File;
use std::io::Write;
use std::collections::HashMap;
use super::super::{Brainfuck, BrainfuckInteger, Instruct};
use super::super::indent::indent;

fn generate_asm_str(out: &mut Write, name: &str, data: &[u8]) -> std::io::Result<()> {
    write!(out, "{:-8}db ", format!("{}:", name))?;
    if data.len() > 0 {
        let indent = " ".to_string()
            .repeat(std::cmp::max(name.len() + 1, 8) + 3)
            .into_bytes();
        let mut quote   = false;
        let mut endline = false;
        let mut first   = true;

        for c in data.iter() {
            match *c {
                b'\n' => {
                    if endline {
                        out.write_all(b", \\\n")?;
                        out.write_all(&indent)?;
                    } else if quote {
                        out.write_all(b"\",")?;
                    } else if !first {
                        out.write_all(b",")?;
                    }
                    out.write_all(b"10")?;
                    endline = true;
                    quote = false;
                },
                b' '..=b'&' | b'('..=b'[' | b']'..=b'~' => {
                    if endline {
                        out.write_all(b", \\\n")?;
                        out.write_all(&indent)?;
                        out.write_all(b"\"")?;
                    } else if quote {
                        // continuing quoted string
                    } else if first {
                        out.write_all(b"\"")?;
                    } else {
                        out.write_all(b",\"")?;
                    }

                    out.write_all(&[*c])?;

                    endline = false;
                    quote = true;
                },
                c => {
                    if endline {
                        out.write_all(b", \\\n")?;
                        out.write_all(&indent)?;
                    } else if quote {
                        out.write_all(b"\",")?;
                    } else if !first {
                        out.write_all(b",")?;
                    }

                    write!(out, "{}", c as u32)?;

                    endline = false;
                    quote = false;
                }
            }
            first = false;
        }

        if quote {
            out.write_all(b"\"\n")?;
        } else {
            out.write_all(b"\n")?;
        }
    } else {
        out.write_all(b"\"\"\n")?;
    }

    return Ok(());
}

fn generate_c_write_str(out: &mut Write, data: &[u8], nesting: usize) -> std::io::Result<()> {
    if data.len() > 0 {
        indent(out, nesting)?;
        let multiline = if let Some(pos) = data.iter().position(|b| *b == b'\n') {
            pos < data.len() - 1
        } else {
            false
        };

        if multiline {
            write!(out, "fwrite(\n")?;
            indent(out, nesting + 1)?;
            write!(out, "\"")?;
        } else {
            write!(out, "fwrite(\"")?;
        }

        for c in data.iter() {
            match *c {
                b'\\' | b'"' => {
                    out.write_all(&[b'\\', *c])?;
                },

                b'\n' => {
                    if multiline {
                        out.write_all(b"\\n\"\n")?;
                        indent(out, nesting + 1)?;
                        write!(out, "\"")?;
                    } else {
                        out.write_all(b"\\n")?;
                    }
                },

                b'\0' => {
                    out.write_all(b"\\0")?;
                },

                b'\r' => {
                    out.write_all(b"\\r")?;
                },

                b'\t' => {
                    out.write_all(b"\\t")?;
                },

                11u8 => {
                    out.write_all(b"\\v")?;
                },

                8u8 => {
                    out.write_all(b"\\b")?;
                },

                c if c >= 32 && c <= 126 => {
                    out.write_all(&[c])?;
                },

                _ => {
                    write!(out, "\\x{:02x}", c)?;
                }
            }
        }

        write!(out, "\", {}, 1, stdout);\n", data.len())?;
    }

    return Ok(());
}

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

            Instruct::WriteStr(_) => {
                last_was_move = false;
            }
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

        write!(runtime, r##"#define _GNU_SOURCE

#include <stdio.h>
#include <stdlib.h>
#include <sys/mman.h>
#include <inttypes.h>
#include <signal.h>
#include <string.h>
#include <unistd.h>

#ifndef __linux__
#   error operating system currently not supported
#endif

#define PAGESIZE {0}

{1}* mem = NULL;
size_t mem_size = 0;
"##, pagesize, Int::c_type())?;

        runtime.write_all(r##"
struct sigaction segv_action;

void brainfuck_main();

void memmng(int signum, siginfo_t *info, void *vctx) {
    (void)signum;

    void *ptr = info->si_addr;
    ucontext_t* ctx = (ucontext_t*)vctx;

    fprintf(stderr, "segfault at index 0x%zu\n", ptr - (void*)mem);

    if (!((ptr >= (void*)mem && ptr < (void*)mem + PAGESIZE) || (ptr >= (void*)mem + (mem_size - PAGESIZE) && ptr < (void*)mem + mem_size))) {
        if (ptr >= (void*)mem + PAGESIZE && ptr < (void*)mem + (mem_size - PAGESIZE)) {
            fprintf(stderr, "pid: %d, bogus SIGSEGV at 0x%zx\n", getpid(), (uintptr_t)ptr);
            abort();
        }
        // Some other segmantation fault! This is a compiler error!
        fprintf(stderr,
            "unhandeled segmantation fault: pagesize = %zu, ptr = 0x%zX (offset %zu), mem = 0x%zX ... 0x%zX (size %zu)\n",
            (size_t)PAGESIZE,
            (uintptr_t)ptr, (uintptr_t)(ptr - (void*)mem),
            (uintptr_t)(void*)mem, (uintptr_t)((void*)mem + mem_size), mem_size);
        fflush(stderr);
        abort();
    }

    if (SIZE_MAX - PAGESIZE < mem_size) {
        fprintf(stderr, "out of address space\n");
        fflush(stderr);
        abort();
    }

    size_t new_size = mem_size + PAGESIZE;
    if (mprotect((void*)mem, PAGESIZE, PROT_READ | PROT_WRITE) != 0) {
        perror("release guard before page protection");
        abort();
    }

    if (mprotect((void*)mem + (mem_size - PAGESIZE), PAGESIZE, PROT_READ | PROT_WRITE) != 0) {
        perror("release guard after page protection");
        abort();
    }

    void *new_mem = mremap((void*)mem, mem_size, new_size, MREMAP_MAYMOVE);
    if (new_mem == MAP_FAILED) {
        perror("mremap");
        abort();
    }

    if (mprotect(new_mem, PAGESIZE, PROT_NONE) != 0) {
        perror("mprotect guard before");
        abort();
    }

    if (mprotect(new_mem + (new_size - PAGESIZE), PAGESIZE, PROT_NONE) != 0) {
        perror("mprotect guard after");
        abort();
    }

    if (ptr < (void*)mem + PAGESIZE) {
        // memory underflow, move everything to the right
        memmove(new_mem + PAGESIZE * 2, (void*)new_mem + PAGESIZE, mem_size - PAGESIZE * 2);
        ptr += PAGESIZE;
    }

    ptr = new_mem + (uintptr_t)(ptr - (void*)mem);

#ifdef __x86_64__
    ctx->uc_mcontext.gregs[REG_R12] = (intptr_t)ptr;
#else
#   error architecture currently not supported
#endif

    mem = new_mem;
    mem_size = new_size;
}

int main() {
    memset(&segv_action, 0, sizeof(struct sigaction));

    segv_action.sa_flags = SA_SIGINFO;
    segv_action.sa_sigaction = memmng;
    if (sigaction(SIGSEGV, &segv_action, NULL) == -1) {
        perror("sigaction");
        return EXIT_FAILURE;
    }

    mem_size = PAGESIZE * 3;
    mem = mmap(NULL, mem_size, PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
    if (mem == MAP_FAILED) {
        perror("mmap");
        return EXIT_FAILURE;
    }

    if (mprotect((void*)mem, PAGESIZE, PROT_NONE) != 0) {
        perror("mprotect guard before");
        return EXIT_FAILURE;
    }

    if (mprotect((void*)mem + (mem_size - PAGESIZE), PAGESIZE, PROT_NONE) != 0) {
        perror("mprotect guard after");
        return EXIT_FAILURE;
    }

    brainfuck_main();

    return 0;
}
"##.as_bytes())?;

        let mut str_table = HashMap::new();
        let mut loop_stack = Vec::new();
        let mut loop_count = 0usize;

        let bf_src_filename = format!("{}.asm", binary_file);
        let mut asm = File::create(&bf_src_filename)?;
        filenames.push(bf_src_filename);

        for instr in code.iter() {
            if let Instruct::WriteStr(data) = instr {
                str_table.insert(data, str_table.len());
            }
        }

        asm.write_all(br##"bits 64
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
        extern mem
        global brainfuck_main
brainfuck_main:
        push rbp
        mov  rbp, rsp
        push r12
        mov  qword  r12 , [rel mem]
        add  qword  r12 , {:8} ; ptr = mem + PAGESIZE;
", pagesize)?;

        let int_size = std::mem::size_of::<Int>() as isize;
        let prefix = match int_size {
            1 => "byte ",
            2 => "word ",
            4 => "dword",
            8 => "qword",
            x => panic!("unsupported cell size: {}", x),
        };
        nesting = 0;
        for instr in code.iter() {
            match *instr {
                Instruct::Move(off) => {
                    if int_size == 1 && off == 1 {
                        write!(asm, "        inc  qword r12             ; {:nesting$}ptr ++;\n", "", nesting = nesting)?;
                    } else if int_size == 1 && off == -1 {
                        write!(asm, "        dec  qword r12             ; {:nesting$}ptr --;\n", "", nesting = nesting)?;
                    } else if off > 0 {
                        let val = off * int_size;
                        write!(asm, "        add  qword  r12 , {:8} ; {:nesting$}ptr  += {};\n", val, "", val, nesting = nesting)?;
                    } else if off != 0 {
                        let val = -off * int_size;
                        write!(asm, "        sub  qword  r12 , {:8} ; {:nesting$}ptr  -= {};\n", val, "", val, nesting = nesting)?;
                    }
                },

                Instruct::Add(val) => {
                    let v = val.i64();
                    if v == 1 {
                        write!(asm, "        inc  {} [r12]           ; {:nesting$}*ptr ++;\n", prefix, "", nesting = nesting)?;
                    } else if v == -1 {
                        write!(asm, "        dec  {} [r12]           ; {:nesting$}*ptr --;\n", prefix, "", nesting = nesting)?;
                    } else if v > 0 {
                        write!(asm, "        add  {} [r12], {:8} ; {:nesting$}*ptr += {};\n", prefix, v, "", v, nesting = nesting)?;
                    } else if v != 0 {
                        write!(asm, "        sub  {} [r12], {:8} ; {:nesting$}*ptr -= {};\n", prefix, -v, "", -v, nesting = nesting)?;
                    }
                },

                Instruct::Set(val) => {
                    write!(asm, "        mov  {} [r12], {:8} ; {:nesting$}*ptr  = {};\n", prefix, val.i64(), "", val.i64(), nesting = nesting)?;
                },

                Instruct::Read => {
                    write!(asm, "        call getchar\n")?;

                    match int_size {
                        8 => write!(asm, "        mov  {} [r12], rax      ; {:nesting$}*ptr = getchar();\n", prefix, "", nesting = nesting)?,
                        4 => write!(asm, "        mov  {} [r12], eax      ; {:nesting$}*ptr = getchar();\n", prefix, "", nesting = nesting)?,
                        2 => write!(asm, "        mov  {} [r12], ax       ; {:nesting$}*ptr = getchar();\n", prefix, "", nesting = nesting)?,
                        1 => write!(asm, "        mov  {} [r12], al       ; {:nesting$}*ptr = getchar();\n", prefix, "", nesting = nesting)?,
                        x => panic!("unsupported cell size: {}", x),
                    }
                },

                Instruct::Write => {
                    write!(asm, "        mov  edi, [r12]\n")?;
                    write!(asm, "        call putchar ; putchar(*ptr)\n")?;
                },

                Instruct::LoopStart(_) => {
                    loop_count += 1;
                    loop_stack.push(loop_count);

                    write!(asm, "        cmp  {} [r12],        0 ; {:nesting$}while (*ptr) {{\n", prefix, "", nesting = nesting)?;
                    write!(asm, "        je   loop_{}_end\n", loop_count)?;
                    write!(asm, "loop_{}_start:\n", loop_count)?;
                    nesting += 4;
                },

                Instruct::LoopEnd(_) => {
                    nesting -= 4;
                    let loop_id = loop_stack.pop().unwrap();

                    write!(asm, "        cmp  {} [r12],        0 ; {:nesting$}}}\n", prefix, "", nesting = nesting)?;
                    write!(asm, "        jne  loop_{}_start\n", loop_count)?;

                    write!(asm, "loop_{}_end:\n", loop_id)?;
                },

                Instruct::WriteStr(ref data) => {
                    let msg_id = str_table.get(data).unwrap();

                    write!(asm, "        mov  rcx, [rel stdout]\n")?;
                    write!(asm, "        mov  edx, 1\n")?;
                    write!(asm, "        mov  esi, {}\n", data.len())?;
                    write!(asm, "        mov  edi, msg{}\n", msg_id)?;
                    write!(asm, "        call fwrite             ; {:nesting$}fwrite(msg{}, {}, 1, stdout);\n", "", msg_id, data.len(), nesting = nesting)?;
                },
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
        write!(out, r##"#include <stdio.h>

int main() {{
"##)?;

        for instr in code.iter() {
            if let Instruct::WriteStr(data) = instr {
                generate_c_write_str(&mut out, data, nesting)?;
            }
        }

        write!(out, r##"
    return 0;
}}"##)?;

        filenames.push(c_filename);
    }

    return Ok(filenames);
}

pub fn compile_c(source_file: &str, object_file: &str, debug: bool, optlevel: u32) -> std::io::Result<()> {
    let mut cmd = std::process::Command::new("gcc");
    let cmd = if debug {
        cmd.arg("-g")
    } else {
        &mut cmd
    };
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
            format!("gcc exited with status {}", code)
        } else {
            "gcc terminated by signal".to_string()
        };
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            message));
    }

    return Ok(());
}

pub fn assemble(source_file: &str, object_file: &str, debug: bool, optlevel: u32) -> std::io::Result<()> {
    let mut cmd = std::process::Command::new("nasm");
    let cmd = if debug {
        cmd.arg("-g")
           .arg("-F")
           .arg("dwarf")
    } else {
        &mut cmd
    };
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
            format!("nasm exited with status {}", code)
        } else {
            "nasm terminated by signal".to_string()
        };
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            message));
    }

    return Ok(());
}

pub fn link(obj_files: &[String], binary_file: &str, debug: bool, optlevel: u32) -> std::io::Result<()> {
    let mut cmd = std::process::Command::new("gcc");
    let cmd = if debug {
        cmd.arg("-g")
    } else {
        &mut cmd
    };
    let status = cmd
        .arg(format!("-O{}", optlevel))
        .arg("-o")
        .arg(binary_file)
        .args(obj_files)
        .status()?;

    if !status.success() {
        let message = if let Some(code) = status.code() {
            format!("gcc exited with status {}", code)
        } else {
            "gcc terminated by signal".to_string()
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

    for filename in obj_files {
        std::fs::remove_file(filename)?;
    }
    return Ok(());
}