// some optimizations:
// <{n}                   ->  ptr -= n
// >{n}                   ->  ptr += n
// -{n}                   -> *ptr -= n
// +{n}                   -> *ptr += n
// +{n}-{m}               -> *ptr += n - m // etc.
// [-{*}]                 -> *ptr  = 0
// [-{*}]+{n}             -> *ptr  = n
// .                      -> putchar(*ptr)
// ,                      -> *ptr = getchar()
// [                      -> while (*ptr) {
// ]                      -> }
// [-{*}]+{n}.[-{*}]+{m}. -> write(STDOUT_FILENO, (unsigned char)[] {n, m}, 2)

extern crate num_traits;
extern crate clap;
use clap::{Arg, App, SubCommand};
use num_traits::Signed;
use std::io::Write;

mod brainfuck;

use brainfuck::{Brainfuck, Error, BrainfuckInteger};
use brainfuck::optimize::Options;

fn main() -> std::result::Result<(), std::io::Error> {
    let matches = App::new("Brainfuck Compiler")
        .version("1.0")
        .author("Mathias Panzenböck")

        .arg(Arg::with_name("cell-size")
            .help("size of a memory cell in bytes (default: 32)")
            .possible_values(&["8", "16", "32", "64"])
            .short("s")
            .long("cell-size")
            .takes_value(true))

        .arg(Arg::with_name("optimizations")
            .help("\
Comma separated list of optimization features:
 * fold ........ fold consecutive + - < > operations
 * set ......... detect value setting
 * add_to ...... detect adding one cell to another
 * write ....... join consecutive writes
 * constexpr ... execute code not dependant on input during compile time
 * deadcode .... eliminate dead code
 * all ......... all optimizations
 * none ........ no optimizations (default)

'-feature' removes the feature. E.g. you can write --opt all,-constexpr
to enable all features except constexpr.
")
            .short("O")
            .long("opt")
            .takes_value(true))

        .arg(Arg::with_name("echo-constexpr")
            .help("print program output while evaluating constant part of program")
            .short("e")
            .long("echo-constexpr")
            .takes_value(false))

        .subcommand(SubCommand::with_name("compile")
            .about("compiles a brainfuck program")

            .arg(Arg::with_name("format")
                .help("\
output formats:
 * source....... C and/or assembler source
 * binary ...... x86 64 Linux binary (default)
 * brainfuck ... brainfuck source
 * debug ....... text representation of internal bytecode
")
                .possible_values(&["source", "binary", "brainfuck", "debug"])
                .short("f")
                .long("format")
                .takes_value(true))

            .arg(Arg::with_name("keep-source")
                .help("Keep generated C and/or assembler source files.")
                .short("k")
                .long("keep-source")
                .takes_value(false))

            .arg(Arg::with_name("debug")
                .help("compile debug build")
                .short("g")
                .long("debug")
                .takes_value(false))

            .arg(Arg::with_name("c-opt-level")
                .help("optimization level passed to the C compiler and assembler")
                .long("c-opt-level")
                .takes_value(true))

            .arg(Arg::with_name("OUTPUT")
                .short("o")
                .long("output")
                .takes_value(true)))

        .subcommand(SubCommand::with_name("exec")
            .about("executes a brainfuck program using an interpreter"))

        .arg(Arg::with_name("INPUT")
            .required(true))

        .get_matches();

    let input = matches.value_of("INPUT").expect("input file is required");
    let mut options = Options::none();

    if let Some(opts) = matches.value_of("optimizations") {
        for opt in opts.split(",") {
            match opt.as_ref() {
                "all" | "+all" | "-none" => {
                    options = Options::all();
                },
                "none" | "+none" | "-all" => {
                    options = Options::none();
                },
                "fold" | "+fold" => {
                    options.fold = true;
                },
                "-fold" => {
                    options.fold = false;
                },
                "set" | "+set" => {
                    options.set = true;
                },
                "-set" => {
                    options.set = false;
                },
                "add_to" | "+add_to" | "addto" | "+addto" => {
                    options.add_to = true;
                },
                "-add_to" | "-addto" => {
                    options.add_to = false;
                },
                "write" | "+write" => {
                    options.write = true;
                },
                "-write" => {
                    options.write = false;
                },
                "deadcode" | "+deadcode" => {
                    options.deadcode = true;
                },
                "-deadcode" => {
                    options.deadcode = false;
                },
                "constexpr" | "+constexpr" => {
                    options.constexpr = true;
                },
                "-constexpr" => {
                    options.constexpr = false;
                },
                "" => {},
                _ => {
                    panic!("illegal optimization: {}", opt);
                }
            }
        }
    }

    let int_size: u32 = matches.value_of("cell-size")
        .unwrap_or("32")
        .parse()
        .expect("cell-size is not a positive integer");

    options.constexpr_echo = matches.is_present("echo-constexpr");

    let res = match matches.subcommand() {
        ("compile", Some(sub)) => {
            let format = sub.value_of("format").unwrap_or("binary");
            let keep_source = sub.is_present("keep-source");
            let debug = sub.is_present("debug");
            let c_opt_level: u32 = sub.value_of("c-opt-level")
                .unwrap_or("0")
                .parse()
                .expect("c-opt-level is positive integer");
            let output = sub.value_of("OUTPUT").unwrap_or(
                match format.as_ref() {
                    "source"    => "a.out",
                    "binary"    => "a.out",
                    "brainfuck" => "out.bf",
                    "debug"     => "out.txt",
                    _           => panic!("unsupported format: {}", format)
                });

            match int_size {
                 8 => compile::< i8>(&input, &output, options, &format, keep_source, debug, c_opt_level),
                16 => compile::<i16>(&input, &output, options, &format, keep_source, debug, c_opt_level),
                32 => compile::<i32>(&input, &output, options, &format, keep_source, debug, c_opt_level),
                64 => compile::<i64>(&input, &output, options, &format, keep_source, debug, c_opt_level),
                _  => panic!("illegal integer size: {}", int_size)
            }
        },
        ("exec", _) => {
            match int_size {
                 8 => exec::< i8>(&input, options),
                16 => exec::<i16>(&input, options),
                32 => exec::<i32>(&input, options),
                64 => exec::<i64>(&input, options),
                _  => panic!("illegal integer size: {}", int_size)
            }
        },
        ("", _) => {
            write!(std::io::stderr(), "A sub-command is required!\n")?;
            std::process::exit(1);
        },
        (cmd, _) => {
            write!(std::io::stderr(), "Illegal sub-command: {}\n", cmd)?;
            std::process::exit(1);
        },
    };

    if let Err(err) = res {
        err.print(&mut std::io::stderr(), &input)?;
        std::process::exit(1);
    }

    Ok(())
}

fn compile<Int: BrainfuckInteger + Signed>(
        input: &str, output: &str, options: Options, format: &str, keep_source: bool, debug: bool, c_opt_level: u32)
        -> std::result::Result<(), Error> {
    let code = Brainfuck::<Int>::from_file(input)?;
    let code = code.optimize(options)?;

    match format {
        "source"    => {
            brainfuck::codegen::linux_x86_64::generate(&code, output)?;
        },
        "binary"    => brainfuck::codegen::linux_x86_64::compile(&code, output, debug, c_opt_level, keep_source)?,
        "brainfuck" => {
            let mut out = std::fs::File::create(output)?;
            code.write_bf(&mut out)?;
        },
        "debug"     => {
            let mut out = std::fs::File::create(output)?;
            code.write_debug(&mut out)?;
        },
        _           => panic!("unsupported format: {}", format),
    }

    Ok(())
}

fn exec<Int: BrainfuckInteger + Signed>(input: &str, options: Options) -> std::result::Result<(), Error> {
    let code = Brainfuck::<Int>::from_file(input)?;
    let code = code.optimize(options)?;
    code.exec()?;
    Ok(())
}