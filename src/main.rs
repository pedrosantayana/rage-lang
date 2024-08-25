use std::{fs::File, io::Read, path::PathBuf};

use clap::{command, Parser};

mod compiler;
mod parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct RageClapCli {
    /// File to compile
    file: PathBuf,

    /// Output path to compiled file
    #[arg(short, long, default_value = "program")]
    output: Option<PathBuf>,
}

fn main() {
    let cli = RageClapCli::parse();

    let mut input_file = File::open(cli.file).unwrap();

    let mut input_str = String::new();

    let my_rand = input_file.read_to_string(&mut input_str).unwrap();

    let r = &my_rand as *const usize;
    let tmp = PathBuf::from(format!("/tmp/{}.o", r as u32));

    compiler::compile(&input_str, &tmp);

    run_linker_linux(cli.output.unwrap_or_default(), &tmp);
}

fn run_linker_linux(output: PathBuf, input: &PathBuf) {
    match std::process::Command::new("ld")
        .args([
            "-o",
            output.to_str().unwrap(),
            "-dynamic-linker",
            "/usr/lib/x86_64-linux-gnu/ld-linux-x86-64.so.2",
            "/usr/lib/x86_64-linux-gnu/crt1.o",
            "/usr/lib/x86_64-linux-gnu/crti.o",
            "-lc",
            input.to_str().unwrap(),
            "/usr/lib/x86_64-linux-gnu/crtn.o",
        ])
        .spawn()
        .unwrap()
        .wait()
    {
        Ok(_) => std::fs::remove_file(input).unwrap(),
        Err(e) => println!("[linker error]: {}", e),
    }
}
