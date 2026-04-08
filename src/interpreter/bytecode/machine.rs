use crate::{
  interpreter::{
    bytecode::{
      instruction::{JitCompiledFunction, JitInstruction, JitTerminalInstruction},
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

pub fn evaluate_function<'a>(
  jit_fn: &'a JitCompiledFunction<'a>,
  args: Vec<Value<'a>>,
  context: &'a impl JitFunctionContext<'a>,
) -> InterpreterResult<Value<'a>> {
  let mut locals = LocalTable::<Value<'a>>::new();
  let mut stack = MachineStack::from_args(args);
  let mut pc = jit_fn.entrypoint();

  loop {
    let block = jit_fn
      .block(pc)
      .ok_or_else(|| InterpreterError::jit_err("block does not exist"))?;
    for instr in block.instructions() {
      match instr {
        JitInstruction::BinaryOp(binary_op) => {
          let rhs = stack.pop_value()?;
          let lhs = stack.pop_value()?;
          stack.push_value(match binary_op {
            BinaryOp::Add => lhs.add(&rhs)?,
            BinaryOp::Sub => lhs.subtract(&rhs)?,
            BinaryOp::Mul => lhs.multiply(&rhs)?,
            BinaryOp::Div => lhs.divide(&rhs)?,
            BinaryOp::Mod => lhs.modulo(&rhs)?,
          });
        }
        JitInstruction::LoadLiteral(literal) => {
          stack.push_value(Value::from_literal(literal)?);
        }
        JitInstruction::LoadGlobal(ident) => {
          stack.push_value(context.resolve_ident(ident)?);
        }
        JitInstruction::LoadLocal(local_id) => {
          stack.push_value(locals.read(*local_id)?.clone());
        }
        JitInstruction::StoreLocal(local_id) => {
          locals.write(*local_id, stack.pop_value()?);
        }
        JitInstruction::Call(call_instr) => {
          let target_fn = stack.pop_value()?.as_jit_function()?;

          let mut args = Vec::new();
          for _ in 0..call_instr.arity() {
            args.push(stack.pop_value()?);
          }
          args.reverse();

          stack.push_value(evaluate_function(target_fn, args, context)?);
        }
        JitInstruction::LoadUnit => stack.push_value(Value::Unit),
      }
    }

    match block.terminator() {
      JitTerminalInstruction::Jump(block_id) => {
        pc = *block_id;
      }
      JitTerminalInstruction::ConditionalJump(cond) => {
        let condition = stack.pop_value()?;
        pc = if condition.is_truthy()? {
          cond.true_target()
        } else {
          cond.false_target()
        };
      }
      JitTerminalInstruction::Return => return stack.pop_value(),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::JitFunctionContext;
  use crate::{
    interpreter::{
      bytecode::{
        instruction::{
          JitInstruction, JitTerminalInstruction,
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

  #[gtest]
  fn ret_returns_none() {
    let code = function_bytecode(vec![block(
      vec![JitInstruction::LoadUnit],
      JitTerminalInstruction::Return,
    )]);
    expect_that!(
      evaluate_function(&code, Vec::new(), &EmptyContext),
      ok(unit_value())
    )
  }

  #[gtest]
  fn ret_with_value_on_empty_stack_errors() {
    let code = function_bytecode(vec![block(vec![], JitTerminalInstruction::Return)]);
    expect_that!(
      evaluate_function(&code, Vec::new(), &EmptyContext),
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
      evaluate_function(&code, Vec::new(), &EmptyContext),
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

    expect_that!(
      evaluate_function(&code, Vec::new(), &EmptyContext),
      ok(i32_value(eq(&1))),
    )
  }

  #[gtest]
  fn global_not_found() {
    let global = Ident::new("x");
    let code = function_bytecode(vec![block(
      vec![JitInstruction::LoadGlobal(&global)],
      JitTerminalInstruction::Return,
    )]);

    expect_that!(
      evaluate_function(&code, Vec::new(), &EmptyContext),
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

    expect_that!(
      evaluate_function(&code, Vec::new(), &EmptyContext),
      ok(i32_value(eq(&1))),
    )
  }
}
