mod compiler;
mod parser;

fn main() {
    compiler::compile(r"
        var asd: i8;
        asd = 81;
        libc_putchar(asd);
    ");
}