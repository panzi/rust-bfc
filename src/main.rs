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

mod brainfuck;

use brainfuck::{Brainfuck, Error, BrainfuckInteger};
use brainfuck::optimize::Options;

fn usage(program: &str) -> ! {
    panic!("Usage: {} [options] <input-file>", program);
}

fn main() -> std::result::Result<(), Error> {
    let args: Vec<String> = std::env::args().collect();
    let program = args[0].clone();
    if args.len() <= 1 {
        usage(&program);
    }

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
optimization features:
 * fold ........ fold consecutive + - < > operations
 * set ......... detect value setting
 * write ....... join consecutive writes
 * constexpr ... execute code not dependant on input during compile time
 * all ......... all optimizations
 * none ........ no optimizations (default)
")
            .number_of_values(1)
            .multiple(true)
            .possible_values(&["fold", "set", "write", "constexpr", "all", "none"])
            .short("O")
            .long("opt")
            .takes_value(true))

        .arg(Arg::with_name("echo-constexpr")
            .help("print program output while evaluating constant part of program")
            .short("e")
            .long("echo-constexpr")
            .takes_value(false))

        .subcommand(SubCommand::with_name("compile")
            .arg(Arg::with_name("format")
                .help("\
output formats:
 * C ........... C source
 * binary ...... compiled binary (default)
 * brainfuck ... brainfuck source
 * debug ....... text representation of internal bytecode
")
                .possible_values(&["C", "binary", "brainfuck", "debug"])
                .short("f")
                .long("format")
                .takes_value(true))

            .arg(Arg::with_name("keep-c-source")
                .short("k")
                .long("keep-c-source")
                .takes_value(false))

            .arg(Arg::with_name("debug")
                .help("compile debug build")
                .short("g")
                .long("debug")
                .takes_value(false))

            .arg(Arg::with_name("c-opt-level")
                .help("optimization level passed to the C compiler")
                .long("c-opt-level")
                .takes_value(true))

            .arg(Arg::with_name("OUTPUT")
                .short("o")
                .long("output")
                .takes_value(true)))

        .subcommand(SubCommand::with_name("exec")
            .help("execute program using interpreter"))

        .arg(Arg::with_name("INPUT")
            .required(true))

        .get_matches();

    let input = matches.value_of("INPUT").expect("input file is required");
    let mut options = Options::none();

    if let Some(opts) = matches.values_of("optimizations") {
        for opt in opts {
            match opt.as_ref() {
                "all" => {
                    options = Options::all();
                },
                "none" => {
                    options = Options::none();
                },
                "fold" => {
                    options.fold = true;
                },
                "set" => {
                    options.set = true;
                },
                "write" => {
                    options.write = true;
                },
                "constexpr" => {
                    options.constexpr = true;
                },
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

    match matches.subcommand() {
        ("compile", Some(sub)) => {
            let format = sub.value_of("format").unwrap_or("binary");
            let keep_c = sub.is_present("keep-c-source");
            let debug = sub.is_present("debug");
            let c_opt_level: u32 = sub.value_of("c-opt-level")
                .unwrap_or("0")
                .parse()
                .expect("c-opt-level is positive integer");
            let output = sub.value_of("OUTPUT").unwrap_or(
                match format.as_ref() {
                    "C"         => "out.c",
                    "binary"    => "a.out",
                    "brainfuck" => "out.bf",
                    "debug"     => "out.txt",
                    _           => panic!("unsupported format: {}", format)
                });

            match int_size {
                 8 => compile::< i8>(&input, &output, options, &format, keep_c, debug, c_opt_level)?,
                16 => compile::<i16>(&input, &output, options, &format, keep_c, debug, c_opt_level)?,
                32 => compile::<i32>(&input, &output, options, &format, keep_c, debug, c_opt_level)?,
                64 => compile::<i64>(&input, &output, options, &format, keep_c, debug, c_opt_level)?,
                _  => panic!("illegal integer size: {}", int_size)
            }
        },
        ("exec", _) => {
            match int_size {
                 8 => exec::< i8>(&input, options)?,
                16 => exec::<i16>(&input, options)?,
                32 => exec::<i32>(&input, options)?,
                64 => exec::<i64>(&input, options)?,
                _  => panic!("illegal integer size: {}", int_size)
            }
        },
        (cmd, _) => panic!("illegal command: {}", cmd),
    }

    Ok(())
}

fn compile<Int: BrainfuckInteger + Signed>(
        input: &str, output: &str, options: Options, format: &str, keep_c: bool, debug: bool, c_opt_level: u32)
        -> std::result::Result<(), Error> {
    let code = Brainfuck::<Int>::from_file(input)?;
    let code = code.optimize(options)?;

    match format {
        "C"         => {
            let mut out = std::fs::File::create(output)?;
            brainfuck::codegen::c::generate(&code, &mut out)?;
        },
        "binary"    => brainfuck::codegen::c::compile(&code, output, debug, c_opt_level, keep_c)?,
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