use crate::{
  interpreter::bytecode::{
    instruction_block_list::{BlockId, BlockList},
    local_table::LocalId,
  },
  parser::{
    ast::binary_expression::BinaryOp,
    token::{ident::Ident, literal::Literal},
  },
};

#[derive(Debug)]
pub enum JitInstruction<'a> {
  // Binary operator. Pops rhs off the stack, pops lhs off the stack,
  // combines the two, and pushes the result on the stack.
  //
  // For example, for (2 - 1), the stack would look like:
  //   top -> 1 (rhs)
  //          2 (lhs)
  BinaryOp(BinaryOp),

  // Push a literal value onto the stack.
  LoadLiteral(&'a Literal),

  // Pushes a global value onto the stack.
  LoadGlobal(&'a Ident),

  // Pushes a local onto the stack.
  LoadLocal(LocalId),

  // Pushes the unit value onto the stack.
  LoadUnit,

  // Pops a value off the stack and stores as a local.
  StoreLocal(LocalId),

  // Pops a function value off the stack,
  // pops `arity` arguments off the stack (in reverse order),
  // and calls the desired function.
  //
  // For example, for f(1, 2), the stack would look like:
  //   top -> f
  //          2
  //          1
  Call(JitCallInstruction),
}

#[derive(Debug)]
pub enum JitTerminalInstruction {
  // Jumps to a block of instructions within the current function.
  Jump(BlockId),

  // Pops a value of the stack, jumps based on its true/false value.
  ConditionalJump(ConditionalJumpTargets),

  // Pops a value off the stack and returns it.
  Return,
}

#[derive(Debug)]
pub struct ConditionalJumpTargets {
  true_target: BlockId,
  false_target: BlockId,
}

impl ConditionalJumpTargets {
  pub fn new(true_target: BlockId, false_target: BlockId) -> Self {
    Self {
      true_target,
      false_target,
    }
  }

  pub fn true_target(&self) -> BlockId {
    self.true_target
  }

  pub fn false_target(&self) -> BlockId {
    self.false_target
  }
}

#[derive(Debug)]
pub struct JitCallInstruction {
  arity: u32,
}

impl JitCallInstruction {
  pub fn with_arity(arity: u32) -> Self {
    Self { arity }
  }

  pub fn arity(&self) -> u32 {
    self.arity
  }
}

// A block is a sequence of instructions that are always executed in order
// (possibly pausing for an external function call)
// followed by a terminal instruction.
#[derive(Debug)]
pub struct JitInstructionBlock<'a> {
  instructions: Vec<JitInstruction<'a>>,
  terminator: JitTerminalInstruction,
}

impl<'a> JitInstructionBlock<'a> {
  pub fn new(instructions: Vec<JitInstruction<'a>>, terminator: JitTerminalInstruction) -> Self {
    Self {
      instructions,
      terminator,
    }
  }

  pub fn instructions(&self) -> &[JitInstruction<'a>] {
    &self.instructions
  }

  pub fn terminator(&self) -> &JitTerminalInstruction {
    &self.terminator
  }
}

// A compiled function, composed logical blocks that contain instruction sequences.
#[derive(Debug)]
pub struct JitCompiledFunction<'a> {
  entrypoint: BlockId,
  blocks: BlockList<JitInstructionBlock<'a>>,
}

impl<'a> JitCompiledFunction<'a> {
  pub fn new(entrypoint: BlockId, blocks: BlockList<JitInstructionBlock<'a>>) -> Self {
    Self { entrypoint, blocks }
  }

  pub fn entrypoint(&self) -> BlockId {
    self.entrypoint
  }

  pub fn block(&self, block_id: BlockId) -> Option<&JitInstructionBlock<'a>> {
    self.blocks.block(block_id)
  }
}

#[cfg(test)]
pub mod matchers {
  use super::{
    ConditionalJumpTargets, JitCallInstruction, JitCompiledFunction, JitInstruction,
    JitInstructionBlock, JitTerminalInstruction,
  };
  use crate::{
    interpreter::bytecode::{instruction_block_list::BlockId, local_table::LocalId},
    parser::{
      ast::binary_expression::BinaryOp,
      token::{ident::Ident, literal::Literal},
    },
  };
  use googletest::prelude::*;

  pub fn binary_op_instruction<'a>(
    op_matcher: impl Matcher<&'a BinaryOp>,
  ) -> impl Matcher<&'a JitInstruction<'a>> {
    pat!(JitInstruction::BinaryOp(op_matcher))
  }

  pub fn load_literal_instruction<'a>(
    literal_matcher: impl Matcher<&'a Literal>,
  ) -> impl Matcher<&'a JitInstruction<'a>> {
    pat!(JitInstruction::LoadLiteral(result_of!(
      |lit: &&'a Literal| *lit,
      literal_matcher
    )))
  }

  pub fn load_unit_instruction<'a>() -> impl Matcher<&'a JitInstruction<'a>> {
    pat!(JitInstruction::LoadUnit)
  }

  pub fn load_global_instruction<'a>(
    ident_matcher: impl Matcher<&'a Ident>,
  ) -> impl Matcher<&'a JitInstruction<'a>> {
    pat!(JitInstruction::LoadGlobal(result_of!(
      |ident: &&'a Ident| *ident,
      ident_matcher
    )))
  }

  pub fn load_local_instruction<'a>(
    local_id_matcher: impl Matcher<&'a LocalId>,
  ) -> impl Matcher<&'a JitInstruction<'a>> {
    pat!(JitInstruction::LoadLocal(local_id_matcher))
  }

  pub fn store_local_instruction<'a>(
    local_id_matcher: impl Matcher<&'a LocalId>,
  ) -> impl Matcher<&'a JitInstruction<'a>> {
    pat!(JitInstruction::StoreLocal(local_id_matcher))
  }

  pub fn call_instruction<'a>(
    call_matcher: impl Matcher<&'a JitCallInstruction>,
  ) -> impl Matcher<&'a JitInstruction<'a>> {
    pat!(JitInstruction::Call(call_matcher))
  }

  pub fn call_with_arity<'a>(
    arity_matcher: impl Matcher<&'a u32>,
  ) -> impl Matcher<&'a JitCallInstruction> {
    pat!(JitCallInstruction {
      arity: arity_matcher
    })
  }

  pub fn conditional_jump_targets<'a>(
    true_target_matcher: impl Matcher<&'a BlockId>,
    false_target_matcher: impl Matcher<&'a BlockId>,
  ) -> impl Matcher<&'a ConditionalJumpTargets> {
    pat!(ConditionalJumpTargets {
      true_target: true_target_matcher,
      false_target: false_target_matcher,
    })
  }

  pub fn if_branch_target<'a>(
    matcher: impl Matcher<&'a BlockId>,
  ) -> impl Matcher<&'a ConditionalJumpTargets> {
    pat!(ConditionalJumpTargets {
      true_target: matcher,
      ..
    })
  }

  pub fn else_branch_target<'a>(
    matcher: impl Matcher<&'a BlockId>,
  ) -> impl Matcher<&'a ConditionalJumpTargets> {
    pat!(ConditionalJumpTargets {
      false_target: matcher,
      ..
    })
  }

  pub fn jump_terminator<'a>(
    target_matcher: impl Matcher<&'a BlockId>,
  ) -> impl Matcher<&'a JitTerminalInstruction> {
    pat!(JitTerminalInstruction::Jump(target_matcher))
  }

  pub fn conditional_jump_terminator<'a>(
    targets_matcher: impl Matcher<&'a ConditionalJumpTargets>,
  ) -> impl Matcher<&'a JitTerminalInstruction> {
    pat!(JitTerminalInstruction::ConditionalJump(targets_matcher))
  }

  pub fn ret_terminator<'a>() -> impl Matcher<&'a JitTerminalInstruction> {
    pat!(JitTerminalInstruction::Return)
  }

  pub fn instruction_block<'a>(
    instructions_matcher: impl Matcher<&'a Vec<JitInstruction<'a>>>,
    terminator_matcher: impl Matcher<&'a JitTerminalInstruction>,
  ) -> impl Matcher<&'a JitInstructionBlock<'a>> {
    pat!(JitInstructionBlock {
      instructions: instructions_matcher,
      terminator: terminator_matcher,
    })
  }

  pub fn has_instruction_block<'a>(
    block_id: BlockId,
    block_matcher: impl Matcher<&'a JitInstructionBlock<'a>>,
  ) -> impl Matcher<&'a JitCompiledFunction<'a>> {
    result_of!(
      move |f: &'a JitCompiledFunction<'a>| f
        .blocks
        .block(block_id)
        .expect("expected present in test"),
      block_matcher
    )
  }

  pub fn entry_block<'a>(
    block_matcher: impl Matcher<&'a JitInstructionBlock<'a>>,
  ) -> impl Matcher<&'a JitCompiledFunction<'a>> {
    all!(result_of!(
      |f: &'a JitCompiledFunction<'a>| f
        .blocks
        .block(f.entrypoint)
        .expect("expected present in test"),
      block_matcher
    ))
  }
}

#[cfg(test)]
pub mod testing {
  use crate::interpreter::bytecode::{
    instruction::{
      JitCompiledFunction, JitInstruction, JitInstructionBlock, JitTerminalInstruction,
    },
    instruction_block_list::testing::{block_id, block_list},
  };

  pub fn block<'a>(
    instructions: Vec<JitInstruction<'a>>,
    terminator: JitTerminalInstruction,
  ) -> JitInstructionBlock<'a> {
    JitInstructionBlock {
      instructions,
      terminator,
    }
  }

  pub fn function_bytecode<'a>(blocks: Vec<JitInstructionBlock<'a>>) -> JitCompiledFunction<'a> {
    JitCompiledFunction {
      entrypoint: block_id(0),
      blocks: block_list(blocks),
    }
  }
}
