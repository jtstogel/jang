use std::fmt::Display;

use cknittel_util::builder::Builder;

use crate::parser::{ast::type_expr::TypeExpression, token::ident::Ident};

#[derive(Clone, Debug)]
pub struct StructuredTypeField {
  name: Ident,
  ty: TypeExpression,
}

impl StructuredTypeField {
  pub fn new(name: Ident, ty: TypeExpression) -> Self {
    Self { name, ty }
  }
}

impl Display for StructuredTypeField {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}: {}", self.name, self.ty)
  }
}

#[derive(Clone, Debug, Builder)]
pub struct StructuredTypeDecl {
  #[vec]
  fields: Vec<StructuredTypeField>,
}

impl StructuredTypeDecl {
  pub fn fields(&self) -> &[StructuredTypeField] {
    &self.fields
  }
}

impl Display for StructuredTypeDecl {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    writeln!(f, "{{")?;
    for field in &self.fields {
      writeln!(f, "{field}")?;
    }
    write!(f, "}}")
  }
}

#[cfg(test)]
pub(crate) mod matchers {
  use crate::parser::{
    ast::{structured_type_decl::StructuredTypeField, type_expr::TypeExpression},
    token::ident::Ident,
  };
  use googletest::prelude::*;

  pub fn type_field<'a>(
    name: impl Matcher<&'a Ident>,
    field_type: impl Matcher<&'a TypeExpression>,
  ) -> impl Matcher<&'a StructuredTypeField> {
    pat!(StructuredTypeField {
      name: name,
      ty: field_type,
    })
  }
}
