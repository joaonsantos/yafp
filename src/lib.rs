use errors::{Error, Result};

use std::collections::HashMap;
use std::fmt::Display;
use std::iter::Peekable;
use std::process::exit;
use std::str::FromStr;

pub mod errors;

/// Represents all possible flag variations.
#[derive(Debug, Clone, Copy)]
enum Flag {
    /// A boolean flag.
    Bool,
    /// A flag which holds a value.
    Value,
}

#[derive(Debug, Clone)]
struct FlagEntry {
    value: Option<String>,
    usage: String,
    typ: Flag,
}

pub struct Parser {
    pub command: String,
    flags: HashMap<String, FlagEntry>,
    required: Vec<String>,
    raw_args: Vec<String>,
}

impl Parser {
    pub fn from_env() -> Self {
        let mut raw_args: Vec<String> = std::env::args().collect();
        let required: Vec<String> = Vec::new();
        Self {
            command: raw_args.remove(0),
            flags: HashMap::new(),
            raw_args,
            required,
        }
    }

    pub fn from_vec(args: Vec<String>) -> Self {
        let mut raw_args = args.clone();
        let required: Vec<String> = Vec::new();
        Self {
            command: raw_args.remove(0),
            flags: HashMap::new(),
            raw_args,
            required,
        }
    }

    pub fn bool_flag(&mut self, flag: &str, usage: &str) {
        self.flags.insert(
            flag.to_string(),
            FlagEntry {
                value: Some("false".to_string()),
                usage: usage.to_string(),
                typ: Flag::Bool,
            },
        );
    }

    pub fn required_flag(&mut self, flag: &str, usage: &str) {
        self.required.push(flag.to_string());
        self.flags.insert(
            flag.to_string(),
            FlagEntry {
                value: None,
                usage: usage.to_string(),
                typ: Flag::Value,
            },
        );
    }

    pub fn optional_flag(&mut self, flag: &str, usage: &str) {
        self.flags.insert(
            flag.to_string(),
            FlagEntry {
                value: None,
                usage: usage.to_string(),
                typ: Flag::Value,
            },
        );
    }

    pub fn get_value<T>(&self, flag: &str) -> Option<T>
    where
        T: FromStr,
        <T as FromStr>::Err: Display,
    {
        match self.flags.get(flag) {
            Some(v) => match &v.value {
                Some(v) => match FromStr::from_str(v) {
                    Ok(v) => Some(v),
                    Err(_) => None,
                },
                None => None,
            },
            None => None,
        }
    }

    pub fn help(&self) -> String {
        let mut help_string = vec![format!("Usage: {} [options...]", self.command)];

        // Ensure help is generated deterministically by sorting the flags.
        let mut flag_keys: Vec<String> = Vec::new();
        for (key, _) in self.flags.iter() {
            flag_keys.push(key.to_string());
        }
        flag_keys.sort();

        for key in flag_keys {
            let flag_entry = self.flags.get(&key).unwrap();
            match flag_entry.typ {
                Flag::Value => {
                    let usage = format!("{} {}", key, "value");
                    help_string.push(format!("  -{}", usage));
                }
                _ => help_string.push(format!("  -{}", key)),
            }
            help_string.push(format!("\t{}", flag_entry.usage));
        }
        let mut help_string = help_string.join("\n");
        help_string.push_str("\n");
        help_string
    }

    fn consume_flag<I>(&mut self, flag: String, it: &mut Peekable<I>) -> Result<()>
    where
        I: Iterator<Item = String>,
    {
        let flag = flag[1..].to_string();
        if self.flags.contains_key(&flag) {
            let arg = self.flags.get(&flag).unwrap();
            match arg.typ {
                Flag::Bool => {
                    self.flags.insert(
                        flag.to_string(),
                        FlagEntry {
                            value: Some("true".to_string()),
                            usage: arg.usage.to_string(),
                            typ: arg.typ,
                        },
                    );
                    Ok(())
                }
                Flag::Value => {
                    let next_token = it.peek();
                    let next_token = match next_token {
                        Some(_) => it.next(),
                        None => None,
                    };
                    match next_token {
                        Some(value) => {
                            self.flags.insert(
                                flag.to_string(),
                                FlagEntry {
                                    value: Some(value.to_string()),
                                    usage: arg.usage.to_string(),
                                    typ: arg.typ,
                                },
                            );
                            Ok(())
                        }
                        None => {
                            return Err(Error::MissingValue(flag));
                        }
                    }
                }
            }
        } else {
            if flag == "help" {
                eprintln!("{}", self.help())
            }
            exit(0);
        }
    }

    fn parse_next<I>(&mut self, it: &mut Peekable<I>) -> Result<Option<String>>
    where
        I: Iterator<Item = String>,
    {
        match it.next() {
            Some(token) => {
                let result = if token.starts_with("-") {
                    match self.consume_flag(token.to_string(), it) {
                        Ok(_) => Ok(None),
                        Err(e) => Err(e),
                    }
                } else {
                    Ok(Some(token.to_string()))
                };
                result
            }
            None => Ok(None),
        }
    }

    pub fn finalize(&mut self) -> Result<Vec<String>> {
        let mut remaining: Vec<String> = Vec::new();

        let raw_args = self.raw_args.clone();
        if raw_args.is_empty() {
            eprintln!("{}", self.help());
            exit(0);
        }

        let mut it = raw_args.iter().cloned().peekable();
        while let Some(_) = it.peek() {
            match self.parse_next(&mut it) {
                Ok(value) => {
                    if let Some(v) = value {
                        remaining.push(v);
                    }
                }
                Err(e) => return Err(e),
            }
        }

        // Check for required flags.
        for flag in &self.required {
            if !self.flags.contains_key(flag) {
                return Err(Error::MissingArgument(flag.to_string()));
            }

            match self.flags.get(flag) {
                Some(entry) => {
                    if let None = entry.value {
                        return Err(Error::MissingArgument(flag.to_string()));
                    }
                }
                None => return Err(Error::MissingArgument(flag.to_string())),
            }
        }
        Ok(remaining)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple() {
        let cmd_args: Vec<String> =
            vec!["head", "-verbose", "-num", "1", "-opt", "val", "file.txt"]
                .iter()
                .map(|x| x.to_string())
                .collect();

        let mut parser = Parser::from_vec(cmd_args);
        parser.bool_flag("verbose", "this is used to get verbose output");
        parser.required_flag("num", "this is used to set a numeric value");
        parser.required_flag("opt", "this is an optional flag (optional)");

        // This must be called before fetching flags and returns any remaining args.
        let mut remaining = parser.finalize().unwrap();
        assert_eq!(remaining.is_empty(), false);
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining.remove(0), "file.txt");

        // Check the value is true as expected.
        let verbose: Option<bool> = parser.get_value("verbose");
        assert_eq!(Some(true), verbose);

        // Check the flag is set as expected.
        let num: Option<i32> = parser.get_value("num");
        assert_eq!(Some(1), num);

        // Check the optional flag is set as expected.
        let opt: Option<String> = parser.get_value("opt");
        assert_eq!(Some(String::from("val")), opt);

        // Check help text generation.
        let help = parser.help();
        assert_eq!(
            help,
            [
                "Usage: head [options...]\n",
                " -num\tthis is used to set a numeric value\n",
                " -opt\tthis is an optional flag (optional)\n",
                " -verbose\tthis is used to get verbose output\n",
            ]
            .concat(),
        )
    }

    #[test]
    fn optional_not_given() {
        let cmd_args: Vec<String> = vec!["head"].iter().map(|x| x.to_string()).collect();

        let mut parser = Parser::from_vec(cmd_args);
        parser.optional_flag("num", "this is used to set a numeric value (optional)");

        // This must be called before fetching flags and returns any remaining args.
        let remaining = parser.finalize().unwrap();
        let remaining = dbg!(remaining);
        assert_eq!(remaining.is_empty(), true);

        // Check the value is not set as expected.
        let num: Option<i32> = parser.get_value("num");
        assert_eq!(None, num);
    }

    #[test]
    fn required_not_given() {
        let cmd_args: Vec<String> = vec!["head", "file.txt"]
            .iter()
            .map(|x| x.to_string())
            .collect();

        let mut parser = Parser::from_vec(cmd_args);
        parser.required_flag("num", "this is used to set a numeric value");

        // This must be called before fetching flags and returns any remaining args.
        let result = parser.finalize();
        assert_eq!(result.is_err(), true);

        // Check the value is not set as expected.
        let num: Option<i32> = parser.get_value("num");
        assert_eq!(None, num);
    }
}
