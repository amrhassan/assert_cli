//! # Test CLI Applications
//!
//! Currently, this crate only includes basic functionality to check the output of a child process
//! is as expected.
//!
//! ## Example
//!
//! Here's a trivial example:
//!
//! ```rust
//! # extern crate assert_cli;
//!
//! assert_cli::assert_cli_output("echo", &["42"], "42").unwrap();
//! ```
//!
//! And here is one that will fail:
//!
//! ```rust,should_panic
//! assert_cli::assert_cli_output("echo", &["42"], "1337").unwrap();
//! ```
//!
//! this will show a nice, colorful diff in your terminal, like this:
//!
//! ```diff
//! -1337
//! +42
//! ```
//!
//! Alternatively, you can use the `assert_cli!` macro:
//!
//! ```rust,ignore
//! assert_cli!("echo 42" => Success, "42").unwrap();
//! ```
//!
//! Make sure to include the crate as `#[macro_use] extern crate assert_cli;`.


#![cfg_attr(feature = "dev", feature(plugin))]
#![cfg_attr(feature = "dev", plugin(clippy))]

#![deny(missing_docs)]

extern crate ansi_term;
extern crate difference;

use std::process::{Command, Output};
use std::error::Error;
use std::ffi::OsStr;

mod cli_error;
mod diff;

use cli_error::CliError;

/// Assert a CLI call returns the expected output.
///
/// To test that
///
/// ```sh
/// ls -n1 src/
/// ```
///
/// returns
///
/// ```plain
/// cli_error.rs
/// diff.rs
/// lib.rs
/// ```
///
/// you would call it like this:
///
/// ```rust,no_run
/// # extern crate assert_cli;
/// assert_cli::assert_cli_output("ls", &["-n1", "src/"], "cli_error.rs\ndiff.rs\nlib.rs");
/// ```
pub fn assert_cli_output<S>(cmd: &str, args: &[S], expected_output: &str) -> Result<(), Box<Error>>
    where S: AsRef<OsStr>
{
    let call: Result<Output, Box<Error>> = Command::new(cmd)
                                               .args(args)
                                               .output()
                                               .map_err(From::from);

    call.and_then(|output| {
            if !output.status.success() {
                return Err(From::from(CliError::WrongExitCode(output)));
            }

            let stdout = String::from_utf8_lossy(&output.stdout);
            let (distance, changes) = difference::diff(expected_output.trim(),
                                                       &stdout.trim(),
                                                       "\n");
            if distance > 0 {
                return Err(From::from(CliError::OutputMissmatch(changes)));
            }

            Ok(())
        })
        .map_err(From::from)
}

/// Assert a CLI call fails with the expected error code and output.
pub fn assert_cli_output_error<S>(cmd: &str,
                                  args: &[S],
                                  error_code: Option<i32>,
                                  expected_output: &str)
                                  -> Result<(), Box<Error>>
    where S: AsRef<OsStr>
{
    let call: Result<Output, Box<Error>> = Command::new(cmd)
                                               .args(args)
                                               .output()
                                               .map_err(From::from);

    call.and_then(|output| {
            if output.status.success() {
                return Err(From::from(CliError::WrongExitCode(output)));
            }

            match (error_code, output.status.code()) {
                (Some(a), Some(b)) if a != b =>
                    return Err(From::from(CliError::WrongExitCode(output))),
                _ => {}
            }

            let stdout = String::from_utf8_lossy(&output.stderr);
            let (distance, changes) = difference::diff(expected_output.trim(),
                                                       &stdout.trim(),
                                                       "\n");
            if distance > 0 {
                return Err(From::from(CliError::OutputMissmatch(changes)));
            }

            Ok(())
        })
        .map_err(From::from)
}

/// The `assert_cli!` macro combines the functionality of the other functions in this crate in one
/// short macro.
///
/// ```rust,ignore
/// assert_cli!("echo 42" => Success, "42").unwrap();
/// assert_cli!("exit 11" => Error 11, "").unwrap();
/// ```
///
/// Make sure to include the crate as `#[macro_use] extern crate assert_cli;`.
#[macro_export]
macro_rules! assert_cli {
    ($cmd:expr, $args:expr => Success, $output:expr) => {{
        $crate::assert_cli_output($cmd, $args, $output)
    }};
    ($cmd:expr, $args:expr => Error, $output:expr) => {{
        $crate::assert_cli_output_error($cmd, $args, None, $output)
    }};
    ($cmd:expr, $args:expr => Error $err:expr, $output:expr) => {{
        $crate::assert_cli_output_error($cmd, $args, Some($err), $output)
    }};
}
