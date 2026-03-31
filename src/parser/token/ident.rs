#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Ident {
  name: String,
}

impl Ident {
  pub fn new(name: impl Into<String>) -> Self {
    Self { name: name.into() }
  }

  pub fn name(&self) -> &str {
    &self.name
  }
}

#[cfg(test)]
pub(crate) mod matchers {
  use crate::parser::token::{JangToken, ident::Ident};
  use googletest::prelude::*;

  pub fn ident(expected_name: &str) -> impl Matcher<&Ident> {
    pat!(Ident {
      name: eq(expected_name),
    })
  }

  pub fn ident_token(expected_name: &str) -> impl Matcher<&JangToken> {
    pat!(JangToken::Ident(ident(expected_name)))
  }
}
