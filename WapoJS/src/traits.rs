use anyhow::Result;
use log::error;
use std::fmt::{Debug, Display};

pub trait ToAnyhowResult<T> {
    fn anyhow(self) -> Result<T>;
}
impl<T, E> ToAnyhowResult<T> for Result<T, E>
where
    E: Display + Debug + Send + Sync + 'static,
{
    fn anyhow(self) -> Result<T> {
        self.map_err(anyhow::Error::msg)
    }
}

pub trait ResultExt {
    fn ignore(self);
    fn log_err(self) -> Self;
}

impl<E: Debug, T> ResultExt for Result<T, E> {
    fn ignore(self) {
        if let Err(err) = self {
            error!("ignored error: {:?}", err);
        }
    }

    fn log_err(self) -> Self {
        if let Err(err) = &self {
            error!("error: {:?}", err);
        }
        self
    }
}
