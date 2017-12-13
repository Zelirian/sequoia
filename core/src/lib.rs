//! Core functionality.

use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// A `&Context` for Sequoia.
///
/// # Example
///
/// A context with reasonable defaults can be created using
/// `Context::new`:
///
/// ```
/// # use sequoia_core::Context;
/// let c = Context::new("org.example.webmail").unwrap();
/// ```
///
/// A context can be configured using the builder pattern with
/// `Context::configure`:
///
/// ```
/// # use sequoia_core::Context;
/// let c = Context::configure("org.example.webmail")
///             .home("/tmp/foo")
///             .build().unwrap();
/// ```
pub struct Context {
    domain: String,
    home: PathBuf,
    lib: PathBuf,
}

/// Returns $PREXIX, or a reasonable default prefix.
fn prefix() -> PathBuf {
    /* XXX: Windows support.  */
    PathBuf::from(option_env!("PREFIX").unwrap_or("/usr/local"))
}

impl Context {
    /// Creates a Context with reasonable defaults.
    ///
    /// `domain` should uniquely identify your application, it is
    /// strongly suggested to use a reversed fully qualified domain
    /// name that is associated with your application.
    pub fn new(domain: &str) -> Result<Self> {
        Self::configure(domain).build()
    }

    /// Creates a Context that can be configured.
    ///
    /// `domain` should uniquely identify your application, it is
    /// strongly suggested to use a reversed fully qualified domain
    /// name that is associated with your application.
    ///
    /// The configuration is seeded like in `Context::new`, but can be
    /// modified.  A configuration has to be finalized using
    /// `.build()` in order to turn it into a Context.
    pub fn configure(domain: &str) -> Config {
        Config(Context {
            domain: String::from(domain),
            home: env::home_dir().unwrap_or(env::temp_dir())
                .join(".sequoia"),
            lib: prefix().join("lib").join("sequoia"),
        })
    }

    /// Returns the domain of the context.
    pub fn domain(&self) -> &str {
        &self.domain
    }

    /// Returns the directory containing shared state.
    pub fn home(&self) -> &Path {
        &self.home
    }

    /// Returns the directory containing backend servers.
    pub fn lib(&self) -> &Path {
        &self.lib
    }
}

/// Represents a `Context` configuration.
///
/// A context can be configured using the builder pattern with
/// `Context::configure`:
///
/// ```
/// # use sequoia_core::Context;
/// let c = Context::configure("org.example.webmail")
///             .home("/tmp/foo")
///             .build().unwrap();
/// ```
pub struct Config(Context);

impl Config {
    /// Finalizes the configuration and returns a `Context`.
    pub fn build(self) -> Result<Context> {
        let c = self.0;
        fs::create_dir_all(c.home())?;
        Ok(c)
    }

    /// Sets the directory containing shared state.
    pub fn home<P: AsRef<Path>>(mut self, home: P) -> Self {
        self.set_home(home);
        self
    }

    /// Sets the directory containing shared state.
    pub fn set_home<P: AsRef<Path>>(&mut self, home: P) {
        self.0.home = PathBuf::new().join(home);
    }

    /// Sets the directory containing backend servers.
    pub fn lib<P: AsRef<Path>>(mut self, lib: P) -> Self {
        self.set_lib(lib);
        self
    }

    /// Sets the directory containing shared state.
    pub fn set_lib<P: AsRef<Path>>(&mut self, lib: P) {
        self.0.lib = PathBuf::new().join(lib);
    }
}

/* Error handling.  */

/// Result type for Sequoia.
pub type Result<T> = ::std::result::Result<T, Error>;

/// Errors for Sequoia.
#[derive(Debug)]
pub enum Error {
    /// An `io::Error` occured.
    IoError(io::Error),
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::IoError(error)
    }
}
