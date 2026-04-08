use std::collections::HashMap;

use crate::{
  interpreter::{
    bytecode::{
      compiler::compile_to_bytecode,
      instruction::JitCompiledFunction,
      machine::{self, JitFunctionContext},
    },
    error::{InterpreterError, InterpreterResult},
    value::Value,
  },
  parser::{ast::jang_file::JangFile, token::ident::Ident},
};

const MAIN_FN_NAME: &str = "main";

struct Program<'a> {
  functions: HashMap<Ident, JitCompiledFunction<'a>>,
}

impl<'a> Program<'a> {
  pub fn lookup_function(&self, var: &Ident) -> Option<&JitCompiledFunction<'a>> {
    self.functions.get(var)
  }
}

pub struct Interpreter<'a> {
  program: Program<'a>,
}

impl<'a> JitFunctionContext<'a> for Interpreter<'a> {
  fn resolve_ident(&'a self, name: &Ident) -> InterpreterResult<Value<'a>> {
    Ok(Value::JitCompiledFunctionRef(
      self
        .program
        .lookup_function(name)
        .ok_or_else(|| InterpreterError::internal_err(format!("unbound variable: {}", name)))?,
    ))
  }
}

impl<'a> Interpreter<'a> {
  pub fn new(jang_file: &'a JangFile) -> InterpreterResult<Self> {
    let function_decls_by_name: HashMap<Ident, JitCompiledFunction<'a>> = jang_file
      .function_decls()
      .iter()
      .map(|f| Ok((f.name().clone(), compile_to_bytecode(f)?)))
      .collect::<InterpreterResult<_>>()?;

    Ok(Interpreter {
      program: Program {
        functions: function_decls_by_name,
      },
    })
  }

  pub fn run_main(&self) -> InterpreterResult<i32> {
    let Some(main_fn) = self.program.functions.get(&Ident::new(MAIN_FN_NAME)) else {
      return Err(InterpreterError::generic_err("main function not found"));
    };

    match machine::evaluate_function(main_fn, Vec::new(), self)? {
      Value::Int32(v) => Ok(v),
      Value::Unit => Err(InterpreterError::value_err("main must return a value")),
      r => Err(InterpreterError::value_err(format!(
        "invalid return value from main: {:?}",
        r
      ))),
    }
  }
}

#[cfg(test)]
mod tests {
  use googletest::{expect_that, gtest, prelude::*};

  use crate::{
    error::JangResult, interpreter::machine::Interpreter,
    parser::grammar::testing::lex_and_parse_jang_file,
  };

  fn interpret_program(text: impl IntoIterator<Item = char>) -> JangResult<i32> {
    let ast = lex_and_parse_jang_file(text).unwrap();
    let interp = Interpreter::new(&ast).unwrap();
    interp.run_main().map_err(|err| err.into())
  }

  #[gtest]
  fn calls_main() {
    expect_that!(
      interpret_program(
        r#"
        fn main() -> i32 {
          ret 1
        }
        "#
        .chars()
      ),
      ok(eq(&1))
    );
  }

  #[gtest]
  fn fn_call() {
    expect_that!(
      interpret_program(
        r#"
        fn other() -> i32 {
          ret 2
        }

        fn main() -> i32 {
          ret 1 + other()
        }
        "#
        .chars()
      ),
      ok(eq(&3))
    );
  }

  #[gtest]
  fn locals() {
    expect_that!(
      interpret_program(
        r#"
        fn add_one(x: i32) -> i32 {
          ret x + 1
        }

        fn main() -> i32 {
          ret add_one(1) + 2 * add_one(2)
        }
        "#
        .chars()
      ),
      ok(eq(&8))
    );
  }

  #[gtest]
  fn integer_arithmetic() {
    expect_that!(
      interpret_program(
        r#"
        fn main() -> i32 {
          ret (8 + 9 % 5) / 2 * (7 - 3) + 6 - 5
        }
        "#
        .chars()
      ),
      ok(eq(&25))
    );
  }

  #[gtest]
  fn unbound_variable_throws_error() {
    expect_that!(
      interpret_program(
        r#"
        fn g() -> i32 {
          ret x
        }
        fn f(x: i32) -> i32 {
          ret g()
        }
        fn main() -> i32 {
          ret f(1)
        }
        "#
        .chars()
      ),
      err(displays_as(contains_substring("unbound variable")))
    );
  }

  #[gtest]
  fn let_bindings() {
    expect_that!(
      interpret_program(
        r#"
        fn main() -> i32 {
          let x = 1
          let y = 2
          ret x + y
        }
        "#
        .chars()
      ),
      ok(eq(&3))
    );
  }

  #[gtest]
  fn divide_by_zero() {
    expect_that!(
      interpret_program(
        r#"
        fn main() -> i32 {
          ret 1 / 0
        }
        "#
        .chars()
      ),
      err(displays_as(contains_substring("division by zero")))
    );
  }

  #[gtest]
  fn if_body() {
    expect_that!(
      interpret_program(
        r#"
        fn main() -> i32 {
          if 1 {
            ret 1
          }
          ret 2
        }
        "#
        .chars()
      ),
      ok(eq(&1))
    );
  }

  #[gtest]
  fn else_body() {
    expect_that!(
      interpret_program(
        r#"
        fn main() -> i32 {
          if 0 {
            ret 1
          } else {
            ret 2
          }
          ret 3
        }
        "#
        .chars()
      ),
      ok(eq(&2))
    );
  }

  #[gtest]
  fn else_if_body() {
    expect_that!(
      interpret_program(
        r#"
        fn main() -> i32 {
          if 0 {
            ret 1
          } else if 1 {
            ret 2
          } else {
            ret 3
          }
          ret 4
        }
        "#
        .chars()
      ),
      ok(eq(&2))
    );
  }

  #[gtest]
  fn else_if_else_body() {
    expect_that!(
      interpret_program(
        r#"
        fn main() -> i32 {
          if 0 {
            ret 1
          } else if 0 {
            ret 2
          } else {
            ret 3
          }
          ret 4
        }
        "#
        .chars()
      ),
      ok(eq(&3))
    );
  }

  #[gtest]
  fn fibonacci() {
    expect_that!(
      interpret_program(
        r#"
        fn fib(n: i32) -> i32 {
          if n - 1 {} else { ret 1 }
          if n {} else { ret 1 }

          ret fib(n - 1) + fib(n - 2)
        }

        fn main() -> i32 {
          ret fib(9)
        }
        "#
        .chars()
      ),
      ok(eq(&55))
    );
  }

  #[gtest]
  fn errors_if_main_does_not_return_a_value() {
    expect_that!(
      interpret_program(
        r#"
        fn main() {}
        "#
        .chars()
      ),
      err(displays_as(contains_substring("main must return a value")))
    );
  }

  #[gtest]
  fn lexical_scope() {
    expect_that!(
      interpret_program(
        r#"
        fn main() -> i32 {
          let x = 1
          {
            let x = 2
            if 1 { ret x }
          }
          ret x
        }
        "#
        .chars()
      ),
      ok(eq(&2))
    );
  }

  #[gtest]
  fn deeply_nested_lexical_scope() {
    expect_that!(
      interpret_program(
        r#"
        fn main() -> i32 {
          let x = 0
          let y = 5
          {
            let x = 2
            { {
                if x { ret y }
            } }
          }
          ret 0
        }
        "#
        .chars()
      ),
      ok(eq(&5))
    );
  }

  #[gtest]
  fn let_in_inner_scope_does_not_affect_outer() {
    expect_that!(
      interpret_program(
        r#"
        fn main() -> i32 {
          let x = 1
          {
            let x = 0
          }
          ret x
        }
        "#
        .chars()
      ),
      ok(eq(&1))
    );
  }
  #[gtest]
  fn function_call_preserves_argument_order_two_args() {
    expect_that!(
      interpret_program(
        r#"
        fn sub(x: i32, y: i32) -> i32 {
          ret x - y
        }

        fn main() -> i32 {
          ret sub(10, 3)
        }
        "#
        .chars()
      ),
      ok(eq(&7))
    );
  }

  #[gtest]
  fn function_call_preserves_argument_order_three_args() {
    expect_that!(
      interpret_program(
        r#"
        fn combine(a: i32, b: i32, c: i32) -> i32 {
          ret a * 100 + b * 10 + c
        }

        fn main() -> i32 {
          ret combine(1, 2, 3)
        }
        "#
        .chars()
      ),
      ok(eq(&123))
    );
  }

  #[gtest]
  fn nested_calls_preserve_argument_order() {
    expect_that!(
      interpret_program(
        r#"
        fn sub(x: i32, y: i32) -> i32 {
          ret x - y
        }

        fn id(x: i32) -> i32 {
          ret x
        }

        fn main() -> i32 {
          ret sub(id(10), id(3))
        }
        "#
        .chars()
      ),
      ok(eq(&7))
    );
  }

  #[gtest]
  fn arithmetic_value_type_mismatch() {
    expect_that!(
      interpret_program(
        r#"
        fn test(x: i32) -> i32 {
          ret x + main
        }

        fn main() -> i32 {
          ret test(1)
        }
        "#
        .chars()
      ),
      err(displays_as(contains_substring("type mismatch")))
    );
  }
}
