use std::{
  convert::Infallible,
  error::Error,
  fmt::{Debug, Display},
};

use cknittel_util::builder::error::BuilderError;
use parser_generator::{ParserUserError, error::ParserError};

use crate::{interpreter::error::InterpreterError, source_location::SourceLocation};

#[derive(Clone)]
pub struct ParseError {
  message: String,
  source_location: SourceLocation,
}

impl ParseError {
  fn new(message: impl Into<String>, source_location: SourceLocation) -> Self {
    Self {
      message: message.into(),
      source_location,
    }
  }
}

impl Display for ParseError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "Parse error: {} (pos: {})",
      self.message,
      self.source_location.pos()
    )
  }
}

impl Debug for ParseError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{self}")
  }
}

#[derive(Clone, ParserUserError)]
#[allow(clippy::enum_variant_names)]
pub enum JangError {
  Parse(ParseError),
  Grammar(ParserError<Infallible>),
  Builder(BuilderError),
  Interpret(InterpreterError),
}

impl JangError {
  pub fn parse_error(message: impl Into<String>, source_location: SourceLocation) -> Self {
    Self::Parse(ParseError::new(message, source_location))
  }
}

impl From<ParserError<JangError>> for JangError {
  fn from(value: ParserError<JangError>) -> Self {
    match value {
      ParserError::UserError(err) => err,
      ParserError::ParseError { message } => {
        JangError::Grammar(ParserError::ParseError { message })
      }
      #[cfg(debug_assertions)]
      ParserError::OverlappingTokenMatchers { token } => {
        JangError::Grammar(ParserError::OverlappingTokenMatchers { token })
      }
    }
  }
}

impl From<BuilderError> for JangError {
  fn from(value: BuilderError) -> Self {
    JangError::Builder(value)
  }
}

impl From<InterpreterError> for JangError {
  fn from(err: InterpreterError) -> Self {
    JangError::Interpret(err)
  }
}

impl Error for JangError {}

impl Display for JangError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Parse(err) => write!(f, "{err}"),
      Self::Grammar(err) => write!(f, "Grammar error: {err}"),
      Self::Builder(err) => write!(f, "Builder error: {err}"),
      Self::Interpret(err) => write!(f, "Interpret error: {err}"),
    }
  }
}

impl Debug for JangError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{self}")
  }
}

pub type JangResult<T = ()> = Result<T, JangError>;
