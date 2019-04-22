extern crate num_traits;

use num_traits::Signed;
use std::fs::File;
use std::io::Write;
use super::super::{Brainfuck, BrainfuckInteger, Instruct};
use super::super::indent::indent;

fn generate_write_str(out: &mut Write, data: &[u8], nesting: usize) -> std::io::Result<()> {
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

pub fn generate<Int: BrainfuckInteger + Signed>(code: &Brainfuck<Int>, out: &mut Write) -> std::io::Result<()> {
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

        write!(out, r##"#define _GNU_SOURCE

#include <stdio.h>
#include <stdlib.h>
#include <sys/mman.h>
#include <inttypes.h>
#include <signal.h>
#include <string.h>

#define PAGESIZE {0}

volatile {1}* mem = NULL;
volatile size_t mem_size = 0;
volatile {1}* ptr = NULL;
"##, pagesize, Int::c_type())?;

        out.write_all(r##"
void memmng(int signum) {
    (void)signum;

    if (!(((void*)ptr >= (void*)mem && (void*)ptr < (void*)mem + PAGESIZE) || ((void*)ptr >= (void*)mem + (mem_size - PAGESIZE) && (void*)ptr < (void*)mem + mem_size))) {
        // Some other segmantation fault! This is a compiler error!
        fprintf(stderr,
            "unhandeled segmantation fault: pagesize = %zu, ptr = 0x%zX (offset %zu), mem = 0x%zX ... 0x%zX (size %zu)\n",
            (size_t)PAGESIZE,
            (uintptr_t)(void*)ptr, (uintptr_t)((void*)ptr - (void*)mem),
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
    if (mprotect((void*)mem + (mem_size - PAGESIZE), PAGESIZE, PROT_READ | PROT_WRITE) != 0) {
        perror("release guard page protection");
        abort();
    }

    void *new_mem = mremap((void*)mem, mem_size, new_size, MREMAP_MAYMOVE);
    if (new_mem == MAP_FAILED) {
        perror("mremap");
        abort();
    }

    if (new_mem != (void*)mem) {
        // memory was moved. not sure if I need to re-protect?
        if (mprotect((void*)new_mem, PAGESIZE, PROT_NONE) != 0) {
            perror("mprotect guard before");
            abort();
        }
    }

    if (mprotect(new_mem + (new_size - PAGESIZE), PAGESIZE, PROT_NONE) != 0) {
        perror("mprotect guard after");
        abort();
    }

    if (ptr < mem) {
        // memory underflow, move everything to the right
        memmove(new_mem + PAGESIZE * 2, (void*)mem + PAGESIZE, mem_size - PAGESIZE * 2);
        ptr = (void*)ptr + PAGESIZE;
    }

    mem = new_mem;
    mem_size = new_size;
}

struct sigaction segv_action;

int main() {
    memset(&segv_action, 0, sizeof(struct sigaction));

    segv_action.sa_handler = memmng;
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

    ptr = (void*)mem + PAGESIZE;

"##.as_bytes())?;

        for instr in code.iter() {
            match instr {
                Instruct::Move(off) => {
                    indent(out, nesting)?;
                    write!(out, "ptr += {};\n", off)?;
                },

                Instruct::Add(val) => {
                    indent(out, nesting)?;
                    write!(out, "*ptr += {:?};\n", val)?;
                },

                Instruct::Set(val) => {
                    indent(out, nesting)?;
                    write!(out, "*ptr = {:?};\n", val)?;
                },

                Instruct::Read => {
                    indent(out, nesting)?;
                    write!(out, "*ptr = getchar();\n")?;
                },

                Instruct::Write => {
                    indent(out, nesting)?;
                    write!(out, "putchar(*ptr);\n")?;
                },

                Instruct::LoopStart(_) => {
                    indent(out, nesting)?;
                    write!(out, "while (*ptr) {{\n")?;
                    nesting += 1;
                },

                Instruct::LoopEnd(_) => {
                    nesting -= 1;
                    indent(out, nesting)?;
                    write!(out, "}}\n")?;
                },

                Instruct::WriteStr(data) => {
                    generate_write_str(out, data, nesting)?;
                },
            }
        }
    } else {
        write!(out, r##"#include <stdio.h>

int main() {{
"##)?;

        for instr in code.iter() {
            if let Instruct::WriteStr(data) = instr {
                generate_write_str(out, data, nesting)?;
            }
        }
    }

    write!(out, r##"
    return 0;
}}"##)?;
    Ok(())
}

pub fn compile_c(source_file: &str, binary_file: &str, debug: bool, optlevel: u32) -> std::io::Result<()> {
    let mut cmd = std::process::Command::new("gcc");
    let cmd = if debug {
        cmd.arg("-g")
    } else {
        &mut cmd
    };
    let cmd = if optlevel > 0 {
        cmd.arg(format!("-O{}", optlevel))
    } else {
        cmd
    };
    let status = cmd
        .arg("-Wall")
        .arg("-Wextra")
        .arg("-std=gnu11")
        .arg("-o")
        .arg(binary_file)
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

pub fn compile<Int: BrainfuckInteger + Signed>(code: &Brainfuck<Int>, binary_file: &str, debug: bool, optlevel: u32, keep_c_source: bool) -> std::io::Result<()> {
    let source_file = format!("{}.c", binary_file);
    {
        let mut file = File::create(&source_file)?;
        generate(code, &mut file)?;
    }
    compile_c(&source_file, &binary_file, debug, optlevel)?;
    if !keep_c_source {
        std::fs::remove_file(&source_file)?;
    }
    return Ok(());
}