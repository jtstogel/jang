use std::collections::HashMap;

use crate::{
  interpreter::{
    bytecode::local_table::LocalId,
    error::{InterpreterError, InterpreterResult},
  },
  parser::token::ident::Ident,
};

#[derive(Debug, Default)]
pub struct JitCompilerLexicalScope<'a>(Box<JitCompilerLexicalBlock<'a>>);

#[derive(Debug, Default)]
struct JitCompilerLexicalBlock<'a> {
  parent: Option<JitCompilerLexicalScope<'a>>,
  next_local_id: LocalId,
  locals: HashMap<&'a Ident, LocalId>,
}

impl<'a> JitCompilerLexicalScope<'a> {
  pub fn get_binding(&self, name: &Ident) -> Option<LocalId> {
    self
      .0
      .locals
      .get(name)
      .copied()
      .or_else(|| self.0.parent.as_ref()?.get_binding(name))
  }

  pub fn bind(&mut self, name: &'a Ident) -> LocalId {
    debug_assert!(!self.0.locals.contains_key(name));

    let local_id = self.0.next_local_id;
    self.0.next_local_id = local_id.next();
    self.0.locals.insert(name, local_id);

    local_id
  }

  pub fn enter_block(self) -> Self {
    let next_local_id = self.0.next_local_id;
    Self(Box::new(JitCompilerLexicalBlock {
      parent: Some(self),
      next_local_id,
      locals: HashMap::new(),
    }))
  }

  pub fn exit_block(self) -> InterpreterResult<Self> {
    self
      .0
      .parent
      .ok_or_else(|| InterpreterError::jit_err("tried to exit from top-level scope"))
  }
}
