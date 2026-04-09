use crate::{
  interpreter::{
    bytecode::{
      instruction::{
        ConditionalJumpTargets, JitCompiledFunction, JitInstruction, JitInstructionBlock,
        JitTerminalInstruction,
      },
      instruction_block_list::BlockId,
      local_table::LocalTable,
    },
    error::{InterpreterError, InterpreterResult},
    value::Value,
  },
  parser::{ast::binary_expression::BinaryOp, token::ident::Ident},
};

#[derive(Default)]
struct MachineStack<'a> {
  items: Vec<Value<'a>>,
}

impl<'a> MachineStack<'a> {
  fn from_args(args: Vec<Value<'a>>) -> Self {
    Self { items: args }
  }

  fn pop_value(&mut self) -> InterpreterResult<Value<'a>> {
    match self.items.pop() {
      Some(v) => Ok(v),
      None => Err(InterpreterError::internal_err("bad stack: empty")),
    }
  }

  fn push_value(&mut self, value: Value<'a>) {
    self.items.push(value);
  }
}

pub trait JitFunctionContext<'a> {
  fn resolve_ident(&'a self, name: &'_ Ident) -> InterpreterResult<Value<'a>>;
}

#[derive(Clone, Copy)]
enum CurrentInstruction<'a> {
  Instr(&'a JitInstruction<'a>),
  Term(&'a JitTerminalInstruction),
}

struct JitInstructionBlockCursor<'a> {
  block: &'a JitInstructionBlock<'a>,
  instr_index: usize,
}

impl<'a> JitInstructionBlockCursor<'a> {
  fn start_of(block: &'a JitInstructionBlock<'a>) -> Self {
    Self {
      block,
      instr_index: 0,
    }
  }

  fn next_instruction(&mut self) -> CurrentInstruction<'a> {
    let idx = self.instr_index;
    self.instr_index += 1;

    debug_assert!(idx <= self.block.instructions().len());
    if let Some(instr) = self.block.instructions().get(idx) {
      CurrentInstruction::Instr(instr)
    } else {
      CurrentInstruction::Term(self.block.terminator())
    }
  }
}

struct JitCallFrame<'a> {
  jit_fn: &'a JitCompiledFunction<'a>,
  locals: LocalTable<Value<'a>>,
  stack: MachineStack<'a>,
  block_cursor: JitInstructionBlockCursor<'a>,
}

// Actions that affect which call frames are on the stack.
enum FrameAction<'a> {
  // Does nothing.
  Continue,

  // Pushes a call frame and transitions execution to target_fn.
  Call {
    target_fn: &'a JitCompiledFunction<'a>,
    args: Vec<Value<'a>>,
  },

  // Pops a call frame and passes its return value to the callee.
  Return(Value<'a>),
}

// A single call frame.
impl<'a> JitCallFrame<'a> {
  fn from_call(
    jit_fn: &'a JitCompiledFunction<'a>,
    args: Vec<Value<'a>>,
  ) -> InterpreterResult<Self> {
    let entrypoint = jit_fn
      .block(jit_fn.entrypoint())
      .ok_or_else(|| InterpreterError::jit_err("entrypoint block must exist"))?;

    Ok(Self {
      jit_fn,
      locals: LocalTable::<Value<'a>>::new(),
      stack: MachineStack::from_args(args),
      block_cursor: JitInstructionBlockCursor::start_of(entrypoint),
    })
  }

  fn next_instruction(&mut self) -> CurrentInstruction<'a> {
    self.block_cursor.next_instruction()
  }

  fn switch_to_block(&mut self, id: BlockId) -> InterpreterResult<()> {
    let block = self
      .jit_fn
      .block(id)
      .ok_or_else(|| InterpreterError::jit_err("target block must exist"))?;
    self.block_cursor = JitInstructionBlockCursor::start_of(block);
    Ok(())
  }

  // Executes a single instruction.
  fn step(
    &mut self,
    context: &'a impl JitFunctionContext<'a>,
  ) -> InterpreterResult<FrameAction<'a>> {
    match self.next_instruction() {
      CurrentInstruction::Instr(instr) => match instr {
        JitInstruction::BinaryOp(op) => {
          self.execute_binary_operation(op)?;
        }
        JitInstruction::LoadLiteral(literal) => {
          self.stack.push_value(Value::from_literal(literal)?);
        }
        JitInstruction::LoadGlobal(ident) => {
          self.stack.push_value(context.resolve_ident(ident)?);
        }
        JitInstruction::LoadLocal(local_id) => {
          self.stack.push_value(self.locals.read(*local_id)?.clone());
        }
        JitInstruction::LoadUnit => {
          self.stack.push_value(Value::Unit);
        }
        JitInstruction::StoreLocal(local_id) => {
          self.locals.write(*local_id, self.stack.pop_value()?);
        }
        JitInstruction::Call(call_instr) => {
          let target_fn = self.stack.pop_value()?.as_jit_function()?;
          let args = self.pop_call_args(call_instr.arity() as usize)?;
          return Ok(FrameAction::Call { target_fn, args });
        }
      },
      CurrentInstruction::Term(term) => match term {
        JitTerminalInstruction::Jump(block_id) => {
          self.switch_to_block(*block_id)?;
        }
        JitTerminalInstruction::ConditionalJump(targets) => {
          self.execute_conditional_jump(targets)?;
        }
        JitTerminalInstruction::Return => {
          return Ok(FrameAction::Return(self.stack.pop_value()?));
        }
      },
    }

    Ok(FrameAction::Continue)
  }

  fn execute_binary_operation(&mut self, op: &BinaryOp) -> InterpreterResult<()> {
    let rhs = self.stack.pop_value()?;
    let lhs = self.stack.pop_value()?;
    let value = match op {
      BinaryOp::Add => lhs.add(&rhs)?,
      BinaryOp::Sub => lhs.subtract(&rhs)?,
      BinaryOp::Mul => lhs.multiply(&rhs)?,
      BinaryOp::Div => lhs.divide(&rhs)?,
      BinaryOp::Mod => lhs.modulo(&rhs)?,
      op => return Err(InterpreterError::unimplemented(format!("{op}"))),
    };
    self.stack.push_value(value);
    Ok(())
  }

  fn execute_conditional_jump(
    &mut self,
    targets: &ConditionalJumpTargets,
  ) -> InterpreterResult<()> {
    let condition = self.stack.pop_value()?;
    let target = if condition.is_truthy()? {
      targets.true_target()
    } else {
      targets.false_target()
    };
    self.switch_to_block(target)
  }

  fn pop_call_args(&mut self, arity: usize) -> InterpreterResult<Vec<Value<'a>>> {
    let mut args = Vec::with_capacity(arity);
    for _ in 0..arity {
      args.push(self.stack.pop_value()?);
    }
    args.reverse();
    Ok(args)
  }
}

struct Machine<'a> {
  parent_frames: Vec<JitCallFrame<'a>>,
  frame: JitCallFrame<'a>,
}

enum MachineStatus<'a> {
  Running,
  Finished(Value<'a>),
}

impl<'a> Machine<'a> {
  fn new(jit_fn: &'a JitCompiledFunction<'a>, args: Vec<Value<'a>>) -> InterpreterResult<Self> {
    Ok(Self {
      parent_frames: Vec::new(),
      frame: JitCallFrame::from_call(jit_fn, args)?,
    })
  }

  fn push_frame(
    &mut self,
    target_fn: &'a JitCompiledFunction<'a>,
    args: Vec<Value<'a>>,
  ) -> InterpreterResult<()> {
    let frame = std::mem::replace(&mut self.frame, JitCallFrame::from_call(target_fn, args)?);
    self.parent_frames.push(frame);
    Ok(())
  }

  // Executes a single instruction.
  fn step(
    &mut self,
    context: &'a impl JitFunctionContext<'a>,
  ) -> InterpreterResult<MachineStatus<'a>> {
    match self.frame.step(context)? {
      FrameAction::Continue => Ok(MachineStatus::Running),
      FrameAction::Call { target_fn, args } => {
        self.push_frame(target_fn, args)?;
        Ok(MachineStatus::Running)
      }
      FrameAction::Return(value) => {
        if let Some(parent) = self.parent_frames.pop() {
          self.frame = parent;
          self.frame.stack.push_value(value);
          Ok(MachineStatus::Running)
        } else {
          Ok(MachineStatus::Finished(value))
        }
      }
    }
  }
}

pub fn evaluate_function<'a>(
  jit_fn: &'a JitCompiledFunction<'a>,
  args: Vec<Value<'a>>,
  context: &'a impl JitFunctionContext<'a>,
) -> InterpreterResult<Value<'a>> {
  let mut machine = Machine::new(jit_fn, args)?;

  loop {
    match machine.step(context)? {
      MachineStatus::Running => {}
      MachineStatus::Finished(value) => return Ok(value),
    }
  }
}

#[cfg(test)]
mod tests {
  use std::collections::HashMap;

  use super::JitFunctionContext;
  use crate::{
    interpreter::{
      bytecode::{
        instruction::{
          JitCallInstruction, JitCompiledFunction, JitInstruction, JitTerminalInstruction,
          testing::{block, function_bytecode},
        },
        local_table::testing::local_id,
        machine::evaluate_function,
      },
      error::{InterpreterError, InterpreterResult},
      value::{
        Value,
        matchers::{i32_value, unit_value},
      },
    },
    parser::{
      ast::binary_expression::BinaryOp,
      token::{
        ident::Ident,
        literal::{Literal, NumericLiteral},
      },
    },
  };
  use googletest::prelude::*;

  struct EmptyContext;

  impl<'a> JitFunctionContext<'a> for EmptyContext {
    fn resolve_ident(&'a self, name: &'_ Ident) -> InterpreterResult<Value<'a>> {
      Err(InterpreterError::generic_err(format!(
        "not found in empty context: {name:?}"
      )))
    }
  }

  #[derive(Default)]
  struct ConstContext<'a> {
    vals: HashMap<Ident, Value<'a>>,
  }

  impl<'a> JitFunctionContext<'a> for ConstContext<'a> {
    fn resolve_ident(&'a self, name: &'_ Ident) -> InterpreterResult<Value<'a>> {
      self.vals.get(name).cloned().ok_or_else(|| {
        InterpreterError::generic_err(format!("not found in test context: {name:?}"))
      })
    }
  }

  fn evaluate_unary_fn<'a>(jit_fn: &'a JitCompiledFunction<'a>) -> InterpreterResult<Value<'a>> {
    evaluate_function(jit_fn, Vec::new(), &EmptyContext)
  }

  #[gtest]
  fn ret_returns_none() {
    let code = function_bytecode(vec![block(
      vec![JitInstruction::LoadUnit],
      JitTerminalInstruction::Return,
    )]);
    expect_that!(evaluate_unary_fn(&code), ok(unit_value()))
  }

  #[gtest]
  fn ret_with_value_on_empty_stack_errors() {
    let code = function_bytecode(vec![block(vec![], JitTerminalInstruction::Return)]);
    expect_that!(
      evaluate_unary_fn(&code),
      err(displays_as(contains_substring("bad stack: empty")))
    )
  }

  #[gtest]
  fn load_uninitialized_local_errors() {
    let code = function_bytecode(vec![block(
      vec![JitInstruction::LoadLocal(local_id(0))],
      JitTerminalInstruction::Return,
    )]);
    expect_that!(
      evaluate_unary_fn(&code),
      err(displays_as(contains_substring("bad local read")))
    )
  }

  #[gtest]
  fn store_then_load_local_round_trips() {
    let one = Literal::Numeric(NumericLiteral::from_str("1"));
    let code = function_bytecode(vec![block(
      vec![
        JitInstruction::LoadLiteral(&one),
        JitInstruction::StoreLocal(local_id(0)),
        JitInstruction::LoadLocal(local_id(0)),
      ],
      JitTerminalInstruction::Return,
    )]);

    expect_that!(evaluate_unary_fn(&code), ok(i32_value(eq(&1))),)
  }

  #[gtest]
  fn global_not_found() {
    let global = Ident::new("x");
    let code = function_bytecode(vec![block(
      vec![JitInstruction::LoadGlobal(&global)],
      JitTerminalInstruction::Return,
    )]);

    expect_that!(
      evaluate_unary_fn(&code),
      err(displays_as(contains_substring(
        "not found in empty context"
      ))),
    )
  }

  #[gtest]
  fn subtraction_uses_rhs_lhs_order() {
    let two = Literal::Numeric(NumericLiteral::from_str("2"));
    let one = Literal::Numeric(NumericLiteral::from_str("1"));
    let code = function_bytecode(vec![block(
      vec![
        JitInstruction::LoadLiteral(&two),
        JitInstruction::LoadLiteral(&one),
        JitInstruction::BinaryOp(BinaryOp::Sub),
      ],
      JitTerminalInstruction::Return,
    )]);

    expect_that!(evaluate_unary_fn(&code), ok(i32_value(eq(&1))),)
  }

  #[gtest]
  fn function_call() {
    let sub_function = function_bytecode(vec![block(
      vec![
        JitInstruction::StoreLocal(local_id(0)),
        JitInstruction::LoadLocal(local_id(0)),
        JitInstruction::StoreLocal(local_id(1)),
        JitInstruction::LoadLocal(local_id(1)),
        JitInstruction::BinaryOp(BinaryOp::Sub),
      ],
      JitTerminalInstruction::Return,
    )]);
    let sub_function_name = Ident::new("sub");
    let context = ConstContext {
      vals: HashMap::from([(
        sub_function_name.clone(),
        Value::JitCompiledFunctionRef(&sub_function),
      )]),
    };

    let two = Literal::Numeric(NumericLiteral::from_str("2"));
    let one = Literal::Numeric(NumericLiteral::from_str("1"));
    let code = function_bytecode(vec![block(
      vec![
        JitInstruction::LoadLiteral(&two),
        JitInstruction::LoadLiteral(&one),
        JitInstruction::LoadGlobal(&sub_function_name),
        JitInstruction::Call(JitCallInstruction::with_arity(2)),
      ],
      JitTerminalInstruction::Return,
    )]);

    expect_that!(
      evaluate_function(&code, Vec::new(), &context),
      ok(i32_value(eq(&1))),
    )
  }
}
