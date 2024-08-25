# Rage Programming Language

[![Rust](https://github.com/pedrosantayana/rage-lang/actions/workflows/rust.yml/badge.svg)](https://github.com/pedrosantayana/rage-lang/actions/workflows/rust.yml)

**Rage** is a compiled programming language designed for low-level systems programming. Developed using Rust, Rage utilizes Cranelift for efficient backend code generation and Pest for flexible parsing.

## Features

- [x] **Low-Level Systems Focus**: Tailored for applications requiring direct hardware access and system resource management.
- [x] **Built with Rust**: Ensures memory safety and concurrency.
- [x] **Cranelift Backend**: Enables efficient and optimized code generation.
- [x] **Pest Parser**: Provides a clear and maintainable language grammar.
- [x] **Linux Linkage via `ld`**: Support for linking with the Linux loader to streamline executable creation.
- [x] **`crt1.o` Support**: Implementing `crt1.o` as the default runtime for Linux systems. Ensure `libc6-dev` is installed.
- [ ] **Libc Compatibility**: Integration with standard C library functions for enhanced interoperability.

## Usage

To start using Rage, follow these steps:

   ```bash
   git clone https://github.com/pedrosantayana/rage-lang.git
   cd rage-lang
   cargo build --release
   export PATH=$PATH:/path/to/rage-lang/target/release
   rage-lang <file> [options]
   ```

## Examples

```
var c: i8;
c = 80;

libc_putchar(c);
```

## License

This project is licensed under the MIT License. See the LICENSE file for more information.
