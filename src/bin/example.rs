use std::process::exit;

use yafp::Parser;

fn main() {
    let mut parser = Parser::from_env();
    parser.bool_flag("verbose", "this is used to get verbose output");
    parser.required_flag("num", "this is a required flag");

    let result = parser.finalize();
    let remaining = match result {
        Ok(remaining) => remaining,
        Err(e) => {
            println!("{}: {}", parser.command, e);
            exit(1);
        }
    };

    print!("\n### args parsed ###\n\n");

    let verbose: bool = parser.get_value("verbose").unwrap();
    println!("verbose: {}", verbose);

    let num: String = parser.get_value("num").unwrap_or(String::new());
    println!("num: {}", num);
    println!("remaining_args: {}", remaining.join(", "));

    print!("\n### help generation ###\n\n");

    println!("{}", parser.help());
}
