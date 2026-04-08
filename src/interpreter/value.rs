use std::{
  fmt::Debug,
  ops::{Div, Rem},
};

use crate::{
  interpreter::{
    bytecode::instruction::JitCompiledFunction,
    error::{InterpreterError, InterpreterResult},
    parse_as::ParseAs,
  },
  parser::token::literal::{Literal, NumericLiteral},
};

#[derive(Clone, Debug)]
pub enum Value<'a> {
  Unit,
  Int32(i32),
  Float32(f32),
  JitCompiledFunctionRef(&'a JitCompiledFunction<'a>),
}

/// A pair of two identically-typed numeric values.
#[derive(Debug, Clone)]
enum NumericValuePair {
  Int32(i32, i32),
  Float32(f32, f32),
}

impl<'a> TryFrom<&Literal> for Value<'a> {
  type Error = InterpreterError;

  fn try_from(value: &Literal) -> InterpreterResult<Self> {
    match value {
      Literal::Numeric(NumericLiteral::Integral(v)) => Ok(Self::Int32(v.parse_as()?)),
      Literal::Numeric(NumericLiteral::Float(v)) => Ok(Self::Float32(v.parse_as()?)),
    }
  }
}

impl<'a> Value<'a> {
  pub fn from_literal(literal: &Literal) -> InterpreterResult<Self> {
    literal.try_into()
  }

  pub fn debug_type_name(&self) -> String {
    match self {
      Self::Int32(_) => "i32".into(),
      Self::Float32(_) => "f32".into(),
      Self::JitCompiledFunctionRef(_) => "<compiled-bytecode>".into(),
      Self::Unit => "unit".into(),
    }
  }

  fn to_numeric_pair(&self, other: &Self) -> InterpreterResult<NumericValuePair> {
    match (self, other) {
      (Self::Int32(a), Self::Int32(b)) => Ok(NumericValuePair::Int32(*a, *b)),
      (Self::Float32(a), Self::Float32(b)) => Ok(NumericValuePair::Float32(*a, *b)),
      (Self::Int32(_), _) | (Self::Float32(_), _) => Err(InterpreterError::value_err(format!(
        "type mismatch: {:?} != {:?}",
        self.debug_type_name(),
        other.debug_type_name()
      ))),
      (Self::JitCompiledFunctionRef(_), _) => Err(InterpreterError::value_err(format!(
        "non-numeric value: {}",
        self.debug_type_name()
      ))),
      (Self::Unit, _) => Err(InterpreterError::value_err("unit value, expected numeric")),
    }
  }

  pub fn multiply(&self, other: &Self) -> InterpreterResult<Self> {
    match self
      .to_numeric_pair(other)
      .map_err(|e| e.prefix("multiply: "))?
    {
      NumericValuePair::Int32(a, b) => Ok(Value::Int32(a * b)),
      NumericValuePair::Float32(a, b) => Ok(Value::Float32(a * b)),
    }
  }

  pub fn add(&self, other: &Self) -> InterpreterResult<Self> {
    match self.to_numeric_pair(other).map_err(|e| e.prefix("add: "))? {
      NumericValuePair::Int32(a, b) => Ok(Value::Int32(a + b)),
      NumericValuePair::Float32(a, b) => Ok(Value::Float32(a + b)),
    }
  }

  pub fn subtract(&self, other: &Self) -> InterpreterResult<Self> {
    match self
      .to_numeric_pair(other)
      .map_err(|e| e.prefix("subtract: "))?
    {
      NumericValuePair::Int32(a, b) => Ok(Value::Int32(a - b)),
      NumericValuePair::Float32(a, b) => Ok(Value::Float32(a - b)),
    }
  }

  pub fn divide(&self, divisor: &Self) -> InterpreterResult<Self> {
    match self
      .to_numeric_pair(divisor)
      .map_err(|e| e.prefix("divide: "))?
    {
      NumericValuePair::Int32(a, b) => Ok(Value::Int32(a.checked_div(b).ok_or_else(|| {
        InterpreterError::value_err(format!("division by zero: {:?} / {:?}", self, divisor))
      })?)),
      NumericValuePair::Float32(a, b) => Ok(Value::Float32(a.div(b))),
    }
  }

  pub fn modulo(&self, divisor: &Self) -> InterpreterResult<Self> {
    match self
      .to_numeric_pair(divisor)
      .map_err(|e| e.prefix("modulo: "))?
    {
      NumericValuePair::Int32(a, b) => Ok(Value::Int32(a.checked_rem(b).ok_or_else(|| {
        InterpreterError::value_err(format!("modulo by zero: {:?} / {:?}", self, divisor))
      })?)),
      NumericValuePair::Float32(a, b) => Ok(Value::Float32(a.rem(b))),
    }
  }

  pub fn is_truthy(&self) -> InterpreterResult<bool> {
    match self {
      Value::Int32(v) => Ok(*v != 0),
      value => Err(InterpreterError::value_err(format!(
        "unexpected value in truthy check: {:?}",
        value
      ))),
    }
  }

  pub fn as_jit_function(&self) -> InterpreterResult<&'a JitCompiledFunction<'a>> {
    match self {
      Value::JitCompiledFunctionRef(jit_compiled_function) => Ok(*jit_compiled_function),
      value => Err(InterpreterError::value_err(format!(
        "expected value to be a JIT-compiled function: {:?}",
        value
      ))),
    }
  }
}

#[cfg(test)]
pub mod matchers {
  use crate::interpreter::value::Value;
  use googletest::prelude::*;

  pub fn i32_value<'a>(matcher: impl Matcher<&'a i32>) -> impl Matcher<&'a Value<'a>> {
    pat!(Value::Int32(matcher))
  }

  pub fn unit_value<'a>() -> impl Matcher<&'a Value<'a>> {
    pat!(Value::Unit)
  }
}
