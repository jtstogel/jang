use std::mem;

use crate::{
  error::{JangError, JangResult},
  interpreter::parse_as::ParseAs,
  parser::token::literal::{Literal, NumericLiteral},
};

#[derive(Debug, Clone)]
pub enum Value {
  Int32(i32),
  Float32(f32),
}

impl TryFrom<&Literal> for Value {
  type Error = JangError;

  fn try_from(value: &Literal) -> JangResult<Self> {
    match value {
      Literal::Numeric(NumericLiteral::Integral(v)) => Ok(Value::Int32(v.parse_as()?)),
      Literal::Numeric(NumericLiteral::Float(v)) => Ok(Value::Float32(v.parse_as()?)),
    }
  }
}

impl Value {
  pub fn multiply(&self, other: &Value) -> JangResult<Value> {
    match (self, other) {
      (Value::Int32(a), Value::Int32(b)) => Ok(Value::Int32(a * b)),
      (Value::Float32(a), Value::Float32(b)) => Ok(Value::Float32(a * b)),
      (Value::Int32(_), _) | (Value::Float32(_), _) => Err(JangError::interpret_error(format!(
        "type mismatch in mulitply: {:?} != {:?}",
        mem::discriminant(self),
        mem::discriminant(other)
      ))),
    }
  }

  pub fn add(&self, other: &Value) -> JangResult<Value> {
    match (self, other) {
      (Value::Int32(a), Value::Int32(b)) => Ok(Value::Int32(a + b)),
      (Value::Float32(a), Value::Float32(b)) => Ok(Value::Float32(a + b)),
      (Value::Int32(_), _) | (Value::Float32(_), _) => Err(JangError::interpret_error(format!(
        "type mismatch in add: {:?} != {:?}",
        mem::discriminant(self),
        mem::discriminant(other)
      ))),
    }
  }

  pub fn subtract(&self, other: &Value) -> JangResult<Value> {
    match (self, other) {
      (Value::Int32(a), Value::Int32(b)) => Ok(Value::Int32(a - b)),
      (Value::Float32(a), Value::Float32(b)) => Ok(Value::Float32(a - b)),
      (Value::Int32(_), _) | (Value::Float32(_), _) => Err(JangError::interpret_error(format!(
        "type mismatch in subtract: {:?} != {:?}",
        mem::discriminant(self),
        mem::discriminant(other)
      ))),
    }
  }

  pub fn divide(&self, other: &Value) -> JangResult<Value> {
    match (self, other) {
      (Value::Int32(a), Value::Int32(b)) => {
        if *b != 0 {
          Ok(Value::Int32(a / b))
        } else {
          Err(JangError::interpret_error(format!(
            "divide by zero: {:?} / {:?}",
            a, b
          )))
        }
      }
      (Value::Float32(a), Value::Float32(b)) => {
        if *b != 0. {
          Ok(Value::Float32(a / b))
        } else {
          Err(JangError::interpret_error(format!(
            "divide by zero: {:?} / {:?}",
            a, b
          )))
        }
      }
      (Value::Int32(_), _) | (Value::Float32(_), _) => Err(JangError::interpret_error(format!(
        "type mismatch in divide: {:?} != {:?}",
        mem::discriminant(self),
        mem::discriminant(other)
      ))),
    }
  }

  pub fn modulo(&self, other: &Value) -> JangResult<Value> {
    match (self, other) {
      (Value::Int32(a), Value::Int32(b)) => {
        if *b != 0 {
          Ok(Value::Int32(a % b))
        } else {
          Err(JangError::interpret_error(format!(
            "divide by zero: {:?} / {:?}",
            a, b
          )))
        }
      }
      (Value::Float32(a), Value::Float32(b)) => {
        if *b != 0. {
          Ok(Value::Float32(a % b))
        } else {
          Err(JangError::interpret_error(format!(
            "divide by zero: {:?} / {:?}",
            a, b
          )))
        }
      }
      (Value::Int32(_), _) | (Value::Float32(_), _) => Err(JangError::interpret_error(format!(
        "type mismatch in modulo: {:?} != {:?}",
        mem::discriminant(self),
        mem::discriminant(other)
      ))),
    }
  }
}
