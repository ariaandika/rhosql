//! an error which can occur in sqlite operation
use crate::sqlite::error::{
    BindError, ConfigureError, DecodeError, OpenError, PrepareError, ResetError, StepError,
    display_error, from,
};

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug)]
pub enum Error {
    /// an error when failed to open a database
    Open(OpenError),
    /// an error when failed to configure database
    Configure(ConfigureError),
    /// an error when failed to create prepared statement
    Prepare(PrepareError),
    /// an error when failed to bind value to parameter
    Bind(BindError),
    /// an error when failed to get the next row
    Step(StepError),
    /// an error when failed to decode value
    Decode(DecodeError),
    /// an error when failed to reset or clear binding prepared statement
    Reset(ResetError),
}

from! {
    Error,
    for OpenError => Open,
    for ConfigureError => Configure,
    for PrepareError => Prepare,
    for BindError => Bind,
    for StepError => Step,
    for DecodeError => Decode,
    for ResetError => Reset
}

display_error! {
    Error,
    #delegate Open Configure Prepare Bind Step Decode Reset
}

