use anyhow::Result;
use log::error;
use std::fmt::Debug;

pub trait ResultExt {
    fn ignore(self);
    fn log_err(self) -> Self;
}

impl<E: Debug, T> ResultExt for Result<T, E> {
    fn ignore(self) {
        if let Err(err) = self {
            error!(target: "js", "ignored error: {:?}", err);
        }
    }

    fn log_err(self) -> Self {
        if let Err(err) = &self {
            error!(target: "js", "error: {:?}", err);
        }
        self
    }
}
