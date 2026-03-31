use crate::parser::ast::function_decl::FunctionDecl;

#[derive(Clone, Debug)]
pub struct JangFile {
  function_decls: Vec<FunctionDecl>,
}

impl JangFile {
  pub fn function_decls(&self) -> &[FunctionDecl] {
    &self.function_decls
  }
}

#[derive(Clone, Debug)]
pub struct JangFileBuilder {
  function_decls: Vec<FunctionDecl>,
}

impl JangFileBuilder {
  pub fn new() -> Self {
    JangFileBuilder {
      function_decls: Vec::new(),
    }
  }

  pub fn add_function_decl(mut self, function_decl: FunctionDecl) -> JangFileBuilder {
    self.function_decls.push(function_decl);
    self
  }

  pub fn build(self) -> JangFile {
    JangFile {
      function_decls: self.function_decls,
    }
  }
}

#[cfg(test)]
pub(crate) mod matchers {

  use crate::parser::ast::{function_decl::FunctionDecl, jang_file::JangFile};
  use googletest::prelude::*;

  pub fn jang_file_functions<'a>(
    function_decls_matcher: impl Matcher<&'a [FunctionDecl]>,
  ) -> impl Matcher<&'a JangFile> {
    property!(&JangFile.function_decls(), function_decls_matcher)
  }

  pub fn jang_file_with_fn<'a>(
    function_decl_matcher: impl Matcher<&'a FunctionDecl> + 'a,
  ) -> impl Matcher<&'a JangFile> {
    jang_file_functions(elements_are![function_decl_matcher])
  }
}
