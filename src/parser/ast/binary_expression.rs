use std::fmt::Display;

use crate::parser::ast::expression::Expression;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinaryOp {
  Add,
  Sub,
  Mul,
  Div,
  Mod,
  Equal,
  GreaterThan,
  GreaterThanEqual,
  LessThan,
  LessThanEqual,
  NotEqual,
}

impl Display for BinaryOp {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}",
      match self {
        Self::Add => "+",
        Self::Sub => "-",
        Self::Mul => "*",
        Self::Div => "/",
        Self::Mod => "%",
        Self::Equal => "==",
        Self::GreaterThan => ">",
        Self::GreaterThanEqual => ">=",
        Self::LessThan => "<",
        Self::LessThanEqual => "<=",
        Self::NotEqual => "!=",
      }
    )
  }
}

#[derive(Clone, Debug)]
pub struct BinaryExpression {
  lhs: Box<Expression>,
  rhs: Box<Expression>,
  op: BinaryOp,
}

impl BinaryExpression {
  pub fn lhs(&self) -> &Expression {
    &self.lhs
  }

  pub fn rhs(&self) -> &Expression {
    &self.rhs
  }

  pub fn op(&self) -> BinaryOp {
    self.op
  }
}

impl BinaryExpression {
  pub fn new(
    lhs: impl Into<Box<Expression>>,
    rhs: impl Into<Box<Expression>>,
    op: BinaryOp,
  ) -> Self {
    Self {
      lhs: lhs.into(),
      rhs: rhs.into(),
      op,
    }
  }
}

impl Display for BinaryExpression {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{} {} {}", self.lhs, self.op, self.rhs)
  }
}

#[cfg(test)]
pub(crate) mod matchers {
  use std::ops::Deref;

  use crate::parser::ast::{
    binary_expression::{BinaryExpression, BinaryOp},
    expression::Expression,
  };
  use googletest::prelude::*;

  pub fn binary_expression<'a>(
    lhs_matcher: impl Matcher<&'a Expression>,
    op: &BinaryOp,
    rhs_matcher: impl Matcher<&'a Expression>,
  ) -> impl Matcher<&'a Expression> {
    pat!(Expression::BinaryExpression(pat!(BinaryExpression {
      lhs: result_of!(Box::deref, lhs_matcher),
      rhs: result_of!(Box::deref, rhs_matcher),
      op: eq(op)
    })))
  }
}
