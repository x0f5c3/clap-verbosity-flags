//! Control `log` or `tracing` level with a `--verbose` flag for your CLI
//!
//! # Examples
//!
//! To get `--quiet` and `--verbose` flags through your entire program, just `flatten`
//! [`Verbosity`]:
//! ```rust,no_run
//! # use clap::Parser;
//! # use clap_verbosity_flag::Verbosity;
//! #
//! # /// Le CLI
//! # #[derive(Debug, Parser)]
//! # struct Cli {
//! #[command(flatten)]
//! verbose: Verbosity,
//! # }
//! ```
//!
//! You can then use this to configure your logger:
//! ```rust,no_run
//! # use clap::Parser;
//! # use clap_verbosity_flag::Verbosity;
//! #
//! # /// Le CLI
//! # #[derive(Debug, Parser)]
//! # struct Cli {
//! #     #[command(flatten)]
//! #     verbose: Verbosity,
//! # }
//! let cli = Cli::parse();
//! env_logger::Builder::new()
//!     .filter_level(cli.verbose.log_level_filter())
//!     .init();
//! ```
//!
//! Or your tracing subscriber:
//! ```rust,no_run
//! # use clap::Parser;
//! # use clap_verbosity_flag::Verbosity;
//! #
//! # /// Le CLI
//! # #[derive(Debug, Parser)]
//! # struct Cli {
//! #     #[command(flatten)]
//! #     verbose: Verbosity,
//! # }
//! let cli = Cli::parse();
//! tracing_subscriber::fmt()
//!     .with_max_level(cli.verbose.tracing_level_filter())
//!     .init();
//! ```
//!
//! # Features
//!
//! - `log` (default) enables the `log`-based logging
//! - `tracing` enables the `tracing`-based logging
//!
//! # Logging
//!
//! ## Log
//!
//! By default, this will only report errors.
//! - `-q` silences output
//! - `-v` show warnings
//! - `-vv` show info
//! - `-vvv` show debug
//! - `-vvvv` show trace
//!
//! You can also customize the default logging level:
//! ```rust,no_run
//! # use clap::Parser;
//! use clap_verbosity_flag::{Verbosity, LogLevel};
//!
//! /// Le CLI
//! #[derive(Debug, Parser)]
//! struct Cli {
//!     #[command(flatten)]
//!     verbose: Verbosity<LogLevel>,
//! }
//! ```
//!
//! Or implement [`LogLevel`] yourself for more control.

#![cfg_attr(docsrs, feature(doc_auto_cfg))]

#[cfg(feature = "tracing")]
use tracing_subscriber::filter::LevelFilter;

#[derive(clap::Args, Debug, Clone)]
pub struct Verbosity<L: LogLevel = ErrorLevel> {
    #[arg(
        long,
        short = 'v',
        action = clap::ArgAction::Count,
        global = true,
        help = L::verbose_help(),
        long_help = L::verbose_long_help(),
    )]
    verbose: u8,

    #[arg(
        long,
        short = 'q',
        action = clap::ArgAction::Count,
        global = true,
        help = L::quiet_help(),
        long_help = L::quiet_long_help(),
        conflicts_with = "verbose",
    )]
    quiet: u8,

    #[arg(skip)]
    phantom: std::marker::PhantomData<L>,
}

impl<L: LogLevel> Verbosity<L> {
    /// Create a new verbosity instance by explicitly setting the values
    pub fn new(verbose: u8, quiet: u8) -> Self {
        Verbosity {
            verbose,
            quiet,
            phantom: std::marker::PhantomData,
        }
    }

    #[cfg(feature = "log")]
    /// Get the log level.
    ///
    /// `None` means all output is disabled.
    pub fn log_level(&self) -> Option<log::Level> {
        level_enum_log(self.verbosity())
    }

    #[cfg(feature = "tracing")]
    pub fn tracing_level(&self) -> LevelFilter {
        level_enum_tracing(self.verbosity())
    }

    #[cfg(feature = "log")]
    /// Get the log level filter.
    pub fn log_level_filter(&self) -> log::LevelFilter {
        level_enum_log(self.verbosity())
            .map(|l| l.to_level_filter())
            .unwrap_or(log::LevelFilter::Off)
    }

    #[cfg(feature = "tracing")]
    pub fn tracing_level_filter(&self) -> LevelFilter {
        level_enum_tracing(self.verbosity())
    }

    /// If the user requested complete silence (i.e. not just no-logging).
    pub fn is_silent(&self) -> bool {
        #[cfg(feature = "log")]
        return self.log_level().is_none();
        #[cfg(all(feature = "tracing", not(feature = "log")))]
        return self.tracing_level() == LevelFilter::OFF;
    }

    fn verbosity(&self) -> i8 {
        #[cfg(feature = "log")]
        return level_value_log(L::default_log()) - (self.quiet as i8) + (self.verbose as i8);
        #[cfg(all(feature = "tracing", not(feature = "log")))]
        return level_value_tracing(L::default_tracing()) - (self.quiet as i8)
            + (self.verbose as i8);
    }
}

#[cfg(feature = "log")]
fn level_value_log(level: Option<log::Level>) -> i8 {
    match level {
        None => -1,
        Some(log::Level::Error) => 0,
        Some(log::Level::Warn) => 1,
        Some(log::Level::Info) => 2,
        Some(log::Level::Debug) => 3,
        Some(log::Level::Trace) => 4,
    }
}

#[cfg(feature = "tracing")]
fn level_value_tracing(level: LevelFilter) -> i8 {
    match level {
        LevelFilter::ERROR => 0,
        LevelFilter::WARN => 1,
        LevelFilter::INFO => 2,
        LevelFilter::DEBUG => 3,
        LevelFilter::TRACE => 4,
        _ => -1,
    }
}

#[cfg(feature = "log")]
fn level_enum_log(verbosity: i8) -> Option<log::Level> {
    match verbosity {
        i8::MIN..=-1 => None,
        0 => Some(log::Level::Error),
        1 => Some(log::Level::Warn),
        2 => Some(log::Level::Info),
        3 => Some(log::Level::Debug),
        4.. => Some(log::Level::Trace),
    }
}

#[cfg(feature = "tracing")]
fn level_enum_tracing(verbosity: i8) -> LevelFilter {
    match verbosity {
        i8::MIN..=-1 => LevelFilter::OFF,
        0 => LevelFilter::ERROR,
        1 => LevelFilter::WARN,
        2 => LevelFilter::INFO,
        3 => LevelFilter::DEBUG,
        4.. => LevelFilter::TRACE,
    }
}

use std::fmt;

impl<L: LogLevel> fmt::Display for Verbosity<L> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.verbosity())
    }
}

pub trait LogLevel {
    #[cfg(feature = "log")]
    fn default_log() -> Option<log::Level>;
    #[cfg(feature = "tracing")]
    fn default_tracing() -> Option<LevelFilter>;

    fn verbose_help() -> Option<&'static str> {
        Some("More output per occurrence")
    }

    fn verbose_long_help() -> Option<&'static str> {
        None
    }

    fn quiet_help() -> Option<&'static str> {
        Some("Less output per occurrence")
    }

    fn quiet_long_help() -> Option<&'static str> {
        None
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct ErrorLevel;

impl LogLevel for ErrorLevel {
    #[cfg(feature = "log")]
    fn default_log() -> Option<log::Level> {
        Some(log::Level::Error)
    }

    #[cfg(feature = "tracing")]
    fn default_tracing() -> Option<LevelFilter> {
        Some(LevelFilter::ERROR)
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct WarnLevel;

impl LogLevel for WarnLevel {
    #[cfg(feature = "log")]
    fn default_log() -> Option<log::Level> {
        Some(log::Level::Warn)
    }

    #[cfg(feature = "tracing")]
    fn default_tracing() -> Option<LevelFilter> {
        Some(LevelFilter::WARN)
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct InfoLevel;

impl LogLevel for InfoLevel {
    #[cfg(feature = "log")]
    fn default_log() -> Option<log::Level> {
        Some(log::Level::Info)
    }
    #[cfg(feature = "tracing")]
    fn default_tracing() -> Option<LevelFilter> {
        Some(LevelFilter::INFO)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn verify_app() {
        #[derive(Debug, clap::Parser)]
        struct Cli {
            #[command(flatten)]
            verbose: Verbosity,
        }

        use clap::CommandFactory;
        Cli::command().debug_assert()
    }
}
