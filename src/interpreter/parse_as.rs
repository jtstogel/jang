use std::{fmt::Debug, str::FromStr};

use crate::error::{JangError, JangResult};

pub trait ParseAs<T> {
  fn parse_as(self) -> JangResult<T>;
}

impl<T, E> ParseAs<T> for &str
where
  E: Debug,
  T: FromStr<Err = E>,
{
  fn parse_as(self) -> JangResult<T> {
    self
      .parse()
      .map_err(|err: <T as FromStr>::Err| JangError::interpret_error(format!("{:?}", err)))
  }
}
