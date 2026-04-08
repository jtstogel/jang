use std::fmt::Display;

use cknittel_util::{builder::Builder, from_variants::FromVariants};

use crate::parser::{ast::structured_type_decl::StructuredTypeDecl, token::ident::Ident};

#[derive(Clone, Debug, FromVariants)]
pub enum EnumVariantType {
  TypeRef(Ident),
  Structured(StructuredTypeDecl),
  Empty,
}

impl Display for EnumVariantType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::TypeRef(name) => write!(f, "{name}"),
      Self::Structured(structured) => write!(f, "{structured}"),
      Self::Empty => Ok(()),
    }
  }
}

#[derive(Clone, Debug)]
pub struct EnumVariant {
  name: Ident,
  ty: EnumVariantType,
}

impl EnumVariant {
  pub fn new(name: Ident, ty: EnumVariantType) -> Self {
    Self { name, ty }
  }
}

impl Display for EnumVariant {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{} {}", self.name, self.ty)
  }
}

#[derive(Clone, Debug, Builder)]
pub struct EnumTypeDecl {
  #[vec]
  variants: Vec<EnumVariant>,
}

impl EnumTypeDecl {
  pub fn variants(&self) -> &[EnumVariant] {
    &self.variants
  }
}

impl Display for EnumTypeDecl {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    writeln!(f)?;
    for variant in &self.variants {
      writeln!(f, "| {variant}")?;
    }
    Ok(())
  }
}

#[cfg(test)]
pub(crate) mod matchers {
  use crate::parser::{
    ast::{
      enum_type_decl::{EnumVariant, EnumVariantType},
      structured_type_decl::{StructuredTypeDecl, StructuredTypeField},
    },
    token::ident::Ident,
  };
  use googletest::prelude::*;

  pub fn enum_ref_type<'a>(name: impl Matcher<&'a Ident>) -> impl Matcher<&'a EnumVariantType> {
    pat!(EnumVariantType::TypeRef(name))
  }

  pub fn enum_structured_type<'a>(
    field_matchers: impl Matcher<&'a [StructuredTypeField]>,
  ) -> impl Matcher<&'a EnumVariantType> {
    pat!(EnumVariantType::Structured(property!(
      &StructuredTypeDecl.fields(),
      field_matchers
    )))
  }

  pub fn enum_variant<'a>(name: impl Matcher<&'a Ident>) -> impl Matcher<&'a EnumVariant> {
    pat!(EnumVariant {
      name: name,
      ty: pat!(EnumVariantType::Empty)
    })
  }

  pub fn enum_variant_with<'a>(
    name: impl Matcher<&'a Ident>,
    type_matcher: impl Matcher<&'a EnumVariantType>,
  ) -> impl Matcher<&'a EnumVariant> {
    pat!(EnumVariant {
      name: name,
      ty: type_matcher
    })
  }
}
