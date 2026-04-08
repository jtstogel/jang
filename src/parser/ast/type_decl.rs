use std::fmt::Display;

use cknittel_util::from_variants::FromVariants;

use crate::parser::{
  ast::{enum_type_decl::EnumTypeDecl, structured_type_decl::StructuredTypeDecl},
  token::ident::Ident,
};

#[derive(Clone, Debug, FromVariants)]
pub enum TypeDeclVariant {
  Structured(StructuredTypeDecl),
  Enum(EnumTypeDecl),
}

impl Display for TypeDeclVariant {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Structured(structured) => write!(f, "{structured}"),
      Self::Enum(enum_decl) => write!(f, "{enum_decl}"),
    }
  }
}

#[derive(Clone, Debug)]
pub struct TypeDecl {
  name: Ident,
  decl: TypeDeclVariant,
}

impl TypeDecl {
  pub fn new(name: Ident, decl: TypeDeclVariant) -> Self {
    Self { name, decl }
  }
}

impl Display for TypeDecl {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    writeln!(f, "type {} = {}", self.name, self.decl)
  }
}

#[cfg(test)]
pub(crate) mod matchers {
  use crate::parser::{
    ast::{
      enum_type_decl::{EnumTypeDecl, EnumVariant},
      structured_type_decl::StructuredTypeField,
      type_decl::{StructuredTypeDecl, TypeDecl, TypeDeclVariant},
    },
    token::ident::Ident,
  };
  use googletest::prelude::*;

  pub fn structured_type<'a>(
    name: impl Matcher<&'a Ident>,
    field_matchers: impl Matcher<&'a [StructuredTypeField]>,
  ) -> impl Matcher<&'a TypeDecl> {
    pat!(TypeDecl {
      name: name,
      decl: pat!(TypeDeclVariant::Structured(property!(
        &StructuredTypeDecl.fields(),
        field_matchers
      ))),
    })
  }

  pub fn enum_type<'a>(
    name: impl Matcher<&'a Ident>,
    variant_matchers: impl Matcher<&'a [EnumVariant]>,
  ) -> impl Matcher<&'a TypeDecl> {
    pat!(TypeDecl {
      name: name,
      decl: pat!(TypeDeclVariant::Enum(property!(
        &EnumTypeDecl.variants(),
        variant_matchers
      ))),
    })
  }
}
