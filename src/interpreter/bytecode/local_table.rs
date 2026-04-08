use crate::interpreter::error::{InterpreterError, InterpreterResult};
use std::default::Default;

#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy, Default)]
pub struct LocalId(usize);

impl LocalId {
  pub fn next(self) -> Self {
    Self(self.0 + 1)
  }
}

pub enum LocalSlot<T> {
  Uninitialized,
  Val(T),
}

pub struct LocalTable<T> {
  slots: Vec<LocalSlot<T>>,
}

impl<T> LocalTable<T> {
  pub fn new() -> Self {
    Self { slots: Vec::new() }
  }

  pub fn read(&self, local_id: LocalId) -> InterpreterResult<&T> {
    match self.slots.get(local_id.0) {
      Some(LocalSlot::Val(value)) => Ok(value),
      Some(LocalSlot::Uninitialized) => Err(InterpreterError::internal_err("uninitialized local")),
      None => Err(InterpreterError::internal_err("bad local read")),
    }
  }

  pub fn write(&mut self, local_id: LocalId, value: T) {
    let index = local_id.0;
    if self.slots.len() <= index {
      self
        .slots
        .resize_with(index + 1, || LocalSlot::Uninitialized);
    }
    self.slots[index] = LocalSlot::Val(value);
  }
}

#[cfg(test)]
pub mod testing {
  use crate::interpreter::bytecode::local_table::LocalId;

  pub fn local_id(index: usize) -> LocalId {
    LocalId(index)
  }
}
