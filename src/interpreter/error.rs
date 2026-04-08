use std::fmt::{Debug, Display};

#[derive(Clone, Debug)]
pub enum InterpreterError {
  Generic(String),
  Internal(String),
  JitCompile(String),
  Unimplemented(String),
  Value(String),
}

impl InterpreterError {
  pub fn generic_err(message: impl Into<String>) -> Self {
    Self::Generic(message.into())
  }

  pub fn internal_err(message: impl Into<String>) -> Self {
    Self::Unimplemented(message.into())
  }

  pub fn jit_err(message: impl Into<String>) -> Self {
    Self::JitCompile(message.into())
  }

  pub fn unimplemented(message: impl Into<String>) -> Self {
    Self::Unimplemented(message.into())
  }

  pub fn value_err(message: impl Into<String>) -> Self {
    Self::Value(message.into())
  }

  pub fn prefix(self, message: impl Into<String>) -> Self {
    match self {
      InterpreterError::Generic(s) => InterpreterError::Generic(message.into() + &s),
      InterpreterError::Internal(s) => InterpreterError::Internal(message.into() + &s),
      InterpreterError::JitCompile(s) => InterpreterError::JitCompile(message.into() + &s),
      InterpreterError::Unimplemented(s) => InterpreterError::Unimplemented(message.into() + &s),
      InterpreterError::Value(s) => InterpreterError::Value(message.into() + &s),
    }
  }
}

impl Display for InterpreterError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      InterpreterError::Generic(s) => write!(f, "Interpreter error: {}", s),
      InterpreterError::Unimplemented(s) => write!(f, "Interpreter error: unimplemented: {}", s),
      InterpreterError::JitCompile(s) => write!(f, "Interpreter error in jit compiler: {}", s),
      InterpreterError::Value(s) => write!(f, "Interpreter error for value: {}", s),
      InterpreterError::Internal(s) => {
        write!(f, "Interpreter internal error (interpreter bug): {}", s)
      }
    }
  }
}

pub type InterpreterResult<T = ()> = Result<T, InterpreterError>;
