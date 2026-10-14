// Minimal stub of the `webbrowser` crate to neutralize opening external browsers.
// It exposes a compatible, minimal API used by callers: open(url) -> Result<(), Error>.
use std::fmt;

#[derive(Debug, Clone)]
pub struct Error;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "webbrowser disabled in this build")
    }
}

impl std::error::Error for Error {}

/// Open the given URL in a web browser.
/// This stub intentionally does nothing and returns Ok(()) to disable online behavior.
pub fn open<T: AsRef<str>>(_url: T) -> Result<(), Error> {
    // noop: external browser invocation disabled by vendored stub
    Ok(())
}

/// Provide a second-name alias used by some callers/crates.
pub fn open_browser<T: AsRef<str>>(_url: T) -> Result<(), Error> {
    Ok(())
}
