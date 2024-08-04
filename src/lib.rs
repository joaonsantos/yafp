/*!
yafp is a non-POSIX cli flag parser with imperative style flag declaration instead of the usual declarative style.

Features:
- Help generation.
- Imperative flag declaration with usage text.
- Supports boolean flags, `false` by default and `true` if set.
- Supports required and optional value flags.
- Values parsed to assigned variable type.

Limitations:
- Only supports short flag style.
- Does not support flag combination, for example, `-fd` is not `-f` and `-d` and is instead a single flag.
- Non-UTF8 arguments are not supported
*/

#![forbid(unsafe_code)]
#![warn(missing_docs)]

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

/// The arguments parser.
pub struct Parser {
    /// The name of the command used in the help string.
    pub command: String,
    flags: HashMap<String, FlagEntry>,
    required: Vec<String>,
    raw_args: Vec<String>,
    help_fn: Option<Box<dyn Fn() -> String>>,
}

impl Parser {
    /// Initializes a [`Parser`] using [`std::env::args`] as input.
    pub fn from_env() -> Self {
        let mut raw_args: Vec<String> = std::env::args().collect();
        let required: Vec<String> = Vec::new();
        Self {
            command: raw_args.remove(0),
            flags: HashMap::new(),
            raw_args,
            required,
            help_fn: None,
        }
    }

    /// Initializes a [`Parser`] using a given vector of strings as input.
    pub fn from_vec(args: Vec<String>) -> Self {
        let mut raw_args = args.clone();
        let required: Vec<String> = Vec::new();
        Self {
            command: raw_args.remove(0),
            flags: HashMap::new(),
            raw_args,
            required,
            help_fn: None,
        }
    }

    /// Defines a boolean flag.
    ///
    /// # Examples
    ///
    /// ## Boolean Flag Set
    /// ```
    /// use yafp::Parser;
    /// use yafp::errors::Error;
    ///
    /// let cmd_args: Vec<String> =
    ///     vec!["head", "-verbose", "file.txt"]
    ///         .iter()
    ///         .map(|x| x.to_string())
    ///         .collect();
    ///
    /// let mut parser = Parser::from_vec(cmd_args);
    /// parser.bool_flag("verbose", "this is used to get verbose output");
    ///
    /// /// This must be called before fetching flags and returns any remaining args.
    /// parser.finalize()?;
    ///
    /// /// Since the verbose flag is set this returns true.
    /// let verbose: Option<bool> = parser.get_value("verbose");
    /// assert_eq!(Some(true), verbose);
    /// # Ok::<(), Error>(())
    /// ```
    ///
    /// ## Boolean Flag Unset
    /// ```
    /// use yafp::Parser;
    /// use yafp::errors::Error;
    ///
    /// let cmd_args: Vec<String> =
    ///     vec!["head", "file.txt"]
    ///         .iter()
    ///         .map(|x| x.to_string())
    ///         .collect();
    ///
    /// let mut parser = Parser::from_vec(cmd_args);
    /// parser.bool_flag("verbose", "this is used to get verbose output");
    ///
    /// /// This must be called before fetching flags and returns any remaining args.
    /// parser.finalize()?;
    ///
    /// /// Since the verbose flag is not set this returns false.
    /// let verbose: Option<bool> = parser.get_value("verbose");
    /// assert_eq!(Some(false), verbose);
    /// # Ok::<(), Error>(())
    /// ```
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

    /// Defines a required flag that accepts a value.
    ///
    /// If the flag is not set then [`crate::Parser::finalize`] returns an error
    /// result of type [`crate::errors::Error::MissingArgument`].
    ///
    /// If the flag is set but no value is given then [`crate::Parser::finalize`] returns an error
    /// result of type [`crate::errors::Error::MissingValue`].
    ///
    /// # Examples
    ///
    /// ## Required Flag Set
    /// ```
    /// use yafp::Parser;
    /// use yafp::errors::Error;
    ///
    /// let cmd_args: Vec<String> =
    ///     vec!["head", "-file", "file.txt"]
    ///         .iter()
    ///         .map(|x| x.to_string())
    ///         .collect();
    ///
    /// let mut parser = Parser::from_vec(cmd_args);
    /// parser.required_flag("file", "this is used to set the path for a file");
    ///
    /// /// This must be called before fetching flags and returns any remaining args.
    /// parser.finalize()?;
    ///
    /// /// Since the flag is set this returns the given file path.
    /// let file: Option<String> = parser.get_value("file");
    /// assert_eq!(Some(String::from("file.txt")), file);
    /// # Ok::<(), Error>(())
    /// ```
    ///
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

    /// Defines an optional flag that accepts a value.
    ///
    /// Similar to [`crate::Parser::required_flag`] but [`crate::Parser::finalize`] will not return
    /// an error result if the flag is missing.
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

    /// Returns the value of a flag.
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

    /// Returns a string with the generated flag information.
    pub fn help_flags(&self) -> String {
        let mut flag_keys: Vec<String> = Vec::new();
        for (key, _) in self.flags.iter() {
            flag_keys.push(key.to_string());
        }
        // Ensure flag help is deterministic by sorting flag names.
        flag_keys.sort();

        let mut flag_help_parts: Vec<String> = Vec::new();
        for key in flag_keys {
            let flag_entry = self.flags.get(&key).unwrap();
            match flag_entry.typ {
                Flag::Value => {
                    let usage = format!("{} {}", key, "value");
                    flag_help_parts.push(format!("  -{}", usage));
                }
                _ => flag_help_parts.push(format!("  -{}", key)),
            }
            flag_help_parts.push(format!("\t{}", flag_entry.usage));
        }
        format!("{}\n", flag_help_parts.join("\n"))
    }

    /// Returns a string with the usage string.
    ///
    /// If you use positional arguments it might be useful to define a custom function
    /// which prints the usage line and then prints the string returned by [`crate::Parser::help_flags`].
    ///
    /// # Examples
    ///
    /// ## Default Help
    /// ```
    /// use yafp::Parser;
    /// use yafp::errors::Error;
    ///
    /// let cmd_args: Vec<String> =
    ///     vec!["head", "-verbose", "file.txt"]
    ///         .iter()
    ///         .map(|x| x.to_string())
    ///         .collect();
    ///
    /// let mut parser = Parser::from_vec(cmd_args);
    /// parser.bool_flag("verbose", "this is used to get verbose output");
    ///
    /// /// This must be called before fetching flags and returns any remaining args.
    /// parser.finalize()?;
    ///
    /// /// Using the default help function does not allow you to specify the positional args but let's you get
    /// /// the basic help working.
    /// let help: String = parser.help();
    /// assert_eq!(String::from("Usage: head [options...]\n  -verbose\n\tthis is used to get verbose output\n"), help);
    /// # Ok::<(), Error>(())
    /// ```
    ///
    /// ## Custom Help
    /// ```
    /// use yafp::Parser;
    /// use yafp::errors::Error;
    ///
    /// let cmd_args: Vec<String> =
    ///     vec!["head", "-verbose", "file.txt"]
    ///         .iter()
    ///         .map(|x| x.to_string())
    ///         .collect();
    ///
    /// let mut parser = Parser::from_vec(cmd_args);
    /// parser.bool_flag("verbose", "this is used to get verbose output");
    ///
    /// let command = parser.command.to_string();
    /// let help_flags = parser.help_flags();
    /// parser.set_help_fn(move || {
    ///   let help_string = format!("Usage: {} [options...] <file>", command);
    ///   format!("{}\n{}", help_string, help_flags)
    /// });
    ///
    /// /// This must be called before fetching flags and returns any remaining args.
    /// parser.finalize()?;
    ///
    /// /// Using the default help function does not allow you to specify the positional args but let's you get
    /// /// the basic help working.
    /// let help: String = parser.help();
    /// assert_eq!(String::from("Usage: head [options...] <file>\n  -verbose\n\tthis is used to get verbose output\n"), help);
    /// # Ok::<(), Error>(())
    /// ```
    ///
    pub fn help(&self) -> String {
        match &self.help_fn {
            Some(f) => f(),
            None => {
                let help_string = format!("Usage: {} [options...]", self.command);
                format!("{}\n{}", help_string, self.help_flags())
            }
        }
    }

    /// Accepts a closure that defines a custom help function, for an example usage check the [custom help example].
    ///
    /// [custom help example]: crate::Parser#custom-help
    pub fn set_help_fn(&mut self, f: impl Fn() -> String + 'static) {
        self.help_fn = Some(Box::new(f));
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

    /// Parses the arguments taking into account all defined flags and returns any remaining
    /// non-flag arguments.
    ///
    /// # Errors
    ///
    /// Depending on the flags set, it returns a variant of [`crate::errors::Error`].
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
