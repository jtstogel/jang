use cknittel_util::iter::CollectResult;

use crate::interpreter::error::{InterpreterError, InterpreterResult};

#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
pub struct BlockId(usize);

#[derive(Debug)]
pub struct BlockList<T> {
  blocks: Vec<T>,
}

impl<T> BlockList<T> {
  pub fn block(&self, id: BlockId) -> Option<&T> {
    self.blocks.get(id.0)
  }
}

pub struct BlockListBuilder<T> {
  next_id: BlockId,
  blocks: Vec<Option<T>>,
}

impl<T> BlockListBuilder<T> {
  pub fn new() -> Self {
    Self {
      next_id: BlockId(0),
      blocks: Vec::new(),
    }
  }

  pub fn allocate_uninitialized(&mut self) -> BlockId {
    let id = self.next_id;
    self.next_id = BlockId(id.0 + 1);
    self.blocks.push(None);
    id
  }

  pub fn set(&mut self, block_id: BlockId, block: T) -> InterpreterResult {
    if self.blocks[block_id.0].is_some() {
      return Err(InterpreterError::internal_err("block already exists"));
    }

    self.blocks[block_id.0] = Some(block);
    Ok(())
  }

  pub fn build(self) -> InterpreterResult<BlockList<T>> {
    Ok(BlockList {
      blocks: self
        .blocks
        .into_iter()
        .map(|b| b.ok_or_else(|| InterpreterError::internal_err("block not initialized")))
        .collect_result_vec()?,
    })
  }
}

#[cfg(test)]
pub mod testing {
  use crate::interpreter::bytecode::instruction_block_list::{BlockId, BlockList};

  pub fn block_id(index: usize) -> BlockId {
    BlockId(index)
  }

  pub fn block_list<T>(blocks: Vec<T>) -> BlockList<T> {
    BlockList { blocks }
  }
}
