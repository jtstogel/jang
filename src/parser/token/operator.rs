use std::fmt::Display;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Op {
  /// '='
  Equal,
  /// ','
  Comma,
  /// '('
  OpenParen,
  /// ')'
  CloseParen,
  /// '{'
  OpenBracket,
  /// '}'
  CloseBracket,
  /// '-'
  Dash,
  /// '<'
  LessThan,
  /// '>'
  GreaterThan,
  /// ':'
  Colon,
  /// '.'
  Dot,
  /// '+'
  Plus,
  /// '*'
  Star,
  /// '/'
  Slash,
  /// '%'
  Percent,
  /// '|'
  Bar,
  /// '!'
  Bang,
}

impl Op {
  pub fn from_char(ch: char) -> Option<Self> {
    match ch {
      '=' => Some(Self::Equal),
      ',' => Some(Self::Comma),
      '(' => Some(Self::OpenParen),
      ')' => Some(Self::CloseParen),
      '{' => Some(Self::OpenBracket),
      '}' => Some(Self::CloseBracket),
      '-' => Some(Self::Dash),
      '<' => Some(Self::LessThan),
      '>' => Some(Self::GreaterThan),
      ':' => Some(Self::Colon),
      '.' => Some(Self::Dot),
      '+' => Some(Self::Plus),
      '*' => Some(Self::Star),
      '/' => Some(Self::Slash),
      '%' => Some(Self::Percent),
      '|' => Some(Self::Bar),
      '!' => Some(Self::Bang),
      _ => None,
    }
  }

  fn to_char(self) -> char {
    match self {
      Self::Equal => '=',
      Self::Comma => ',',
      Self::OpenParen => '(',
      Self::CloseParen => ')',
      Self::OpenBracket => '{',
      Self::CloseBracket => '}',
      Self::Dash => '-',
      Self::LessThan => '<',
      Self::GreaterThan => '>',
      Self::Colon => ':',
      Self::Dot => '.',
      Self::Plus => '+',
      Self::Star => '*',
      Self::Slash => '/',
      Self::Percent => '%',
      Self::Bar => '|',
      Self::Bang => '!',
    }
  }

  /// Returns true if this operator can join with `other_op`. This should
  /// always return false for non-operator `other_op`. Joint operators are
  /// treated differently from pairs of separated ones (e.g. "==" vs. "= =").
  pub fn can_join(&self, other_op: char) -> bool {
    match self {
      Self::Dash => other_op == '>',
      Self::CloseParen => other_op == '(',
      Self::Bang => other_op == '=',
      Self::LessThan => other_op == '=',
      Self::GreaterThan => other_op == '=',
      Self::Equal => other_op == '=',
      Self::Comma
      | Self::OpenParen
      | Self::OpenBracket
      | Self::CloseBracket
      | Self::Colon
      | Self::Dot
      | Self::Plus
      | Self::Star
      | Self::Slash
      | Self::Percent
      | Self::Bar => false,
    }
  }
}

impl Display for Op {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.to_char())
  }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Operator {
  pub op: Op,
}

impl Operator {
  pub fn new(op: Op) -> Operator {
    Self { op }
  }

  pub fn op(&self) -> Op {
    self.op
  }
}

#[cfg(test)]
pub(crate) mod matchers {
  use crate::parser::token::{
    JangToken,
    operator::{Op, Operator},
  };
  use googletest::prelude::*;

  pub fn operator_matcher(op: &Op) -> impl Matcher<&JangToken> {
    pat!(JangToken::Operator(pat!(Operator { op: op })))
  }

  #[macro_export]
  macro_rules! operator {
    ($op:ident) => {
      $crate::parser::token::operator::matchers::operator_matcher(
        &$crate::parser::token::operator::Op::$op,
      )
    };
  }
}
