# yafp
![Build Status](https://github.com/joaonsantos/yafp/workflows/CI/badge.svg)
[![Crates.io](https://img.shields.io/crates/v/yafp.svg)](https://crates.io/crates/yafp)
[![Documentation](https://docs.rs/yafp/badge.svg)](https://docs.rs/yafp)
[![Rust 2021](https://img.shields.io/badge/rust-2021-green.svg)](https://www.rust-lang.org)

yafp is a non-POSIX cli flag parser with imperative style flag declaration instead of the usual declarative style. 

Features:
- Help generation.
- Imperative flag declaration with usage text.
- Supports boolean flags, `false` if not found `true` if found.
- Supports required and optional value flags.
- Values parsed to assigned variable type.

Limitations:
- Only supports short flag style.
- Does not support flag combination, for example, `-fd` is not `-f` and `-d` and is instead a single flag.
- Non-UTF8 arguments are not supported.

## Usage

```rs
use yafp::Parser;

fn main() {
    let mut parser = Parser::from_env();
    
    // Declare flags.
    parser.bool_flag("verbose", "this is used to get verbose output");
    parser.required_flag("url", "this is a required flag");
    parser.required_flag("workers", "this is an optional flag");

    // finalize() must be called before accessing arguments.
    // Unbound args are returned if any.
    //
    // An error is returned if there is a parsing error.
    let result = parser.finalize();
    let remaining = match result {
        Ok(remaining) => remaining,
        Err(e) => {
            println!("{}: {}", parser.command, e);
            exit(1);
        }
    };
    
    // yafp parses values to the correct type.
    let verbose: bool = parser.get_value("verbose").unwrap();
    
    //...
}
```

## License

MIT