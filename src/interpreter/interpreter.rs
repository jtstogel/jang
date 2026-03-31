use std::collections::HashMap;

use crate::{
  error::{JangError, JangResult},
  interpreter::value::Value,
  parser::{
    ast::{
      binary_expression::BinaryOp, expression::Expression, function_decl::FunctionDecl,
      jang_file::JangFile, statement::RetStatement,
    },
    token::ident::Ident,
  },
};

const MAIN_FN_NAME: &str = "main";

#[derive(Clone)]
struct LocalScope {
  values: HashMap<Ident, Value>,
}

impl LocalScope {
  fn new() -> Self {
    LocalScope {
      values: HashMap::new(),
    }
  }

  fn lookup(&self, var: &Ident) -> Option<&Value> {
    self.values.get(var)
  }

  fn set_binding(&mut self, var: &Ident, value: Value) {
    self.values.insert(var.clone(), value);
  }
}

struct ExecutionScope {
  parent_scopes: Vec<LocalScope>,
  current_scope: LocalScope,
}

impl ExecutionScope {
  pub fn new() -> Self {
    ExecutionScope {
      parent_scopes: Vec::new(),
      current_scope: LocalScope::new(),
    }
  }

  pub fn lookup_binding(&self, var: &Ident) -> Option<&Value> {
    if let Some(val) = self.current_scope.lookup(var) {
      Some(val)
    } else {
      self
        .parent_scopes
        .iter()
        .rev()
        .map(|scope| scope.lookup(var))
        .find(|val| val.is_some())
        .flatten()
    }
  }

  pub fn bind(&mut self, var: &Ident, value: Value) {
    self.current_scope.set_binding(var, value);
  }

  pub fn push_scope(mut self) -> ExecutionScope {
    let current = self.current_scope;
    self.parent_scopes.push(current);
    self.current_scope = LocalScope::new();
    self
  }

  pub fn pop_scope(mut self) -> ExecutionScope {
    if let Some(scope) = self.parent_scopes.pop() {
      self.current_scope = scope;
    }
    self
  }
}

struct Program {
  functions: HashMap<Ident, FunctionDecl>,
}

impl Program {
  pub fn lookup_function(&self, var: &Ident) -> Option<&FunctionDecl> {
    self.functions.get(var)
  }
}

struct Interpreter {
  program: Program,
}

impl Interpreter {
  fn new(jang_file: JangFile) -> Self {
    let function_decls_by_name: HashMap<Ident, FunctionDecl> = jang_file
      .function_decls()
      .iter()
      .map(|f| (f.name().clone(), f.clone()))
      .collect();

    Interpreter {
      program: Program {
        functions: function_decls_by_name,
      },
    }
  }

  fn run(&self) -> JangResult<Value> {
    self.execute_function(&Ident::new(MAIN_FN_NAME), Vec::new())
  }

  fn execute_function(&self, name: &Ident, parameters: Vec<Value>) -> JangResult<Value> {
    let Some(fn_decl) = self.program.lookup_function(name) else {
      return Err(JangError::interpret_error(format!(
        "function not defined: {:?}",
        name
      )));
    };

    let mut scope = ExecutionScope::new();
    if fn_decl.parameters().len() != parameters.len() {
      return Err(JangError::interpret_error(format!(
        "incorrect number of arguments passed to {:?}",
        name,
      )));
    }
    for (param, value) in fn_decl.parameters().iter().zip(parameters) {
      scope.bind(param.name(), value);
    }

    for stmt in fn_decl.body().statements() {
      match stmt {
        crate::parser::ast::statement::NonRetStatement::Let(let_statement) => {
          scope.bind(
            let_statement.var(),
            self.execute_expression(&scope, let_statement.expr())?,
          );
        }
        crate::parser::ast::statement::NonRetStatement::Block(non_ret_block) => {
          return Err(JangError::interpret_error(
            "blocks in functions not yet supported",
          ));
        }
      }
    }

    let Some(RetStatement::Ret(ret_expr)) = fn_decl.body().ret_statement() else {
      return Err(JangError::interpret_error(
        "functions without return values are not yet supported",
      ));
    };

    self.execute_expression(&scope, ret_expr.expr())
  }

  fn execute_expression(&self, state: &ExecutionScope, expr: &Expression) -> JangResult<Value> {
    match expr {
      Expression::Literal(literal) => Ok(literal.try_into()?),
      Expression::Ident(ident) => {
        if let Some(value) = state.lookup_binding(ident) {
          Ok(value.clone())
        } else {
          Err(JangError::interpret_error(format!(
            "binding for {:?} not found",
            ident
          )))
        }
      }
      Expression::BinaryExpression(binary_expression) => {
        let lhs = self.execute_expression(state, binary_expression.lhs())?;
        let rhs = self.execute_expression(state, binary_expression.rhs())?;
        match binary_expression.op() {
          BinaryOp::Add => lhs.add(&rhs),
          BinaryOp::Sub => lhs.subtract(&rhs),
          BinaryOp::Mul => lhs.multiply(&rhs),
          BinaryOp::Div => lhs.divide(&rhs),
          BinaryOp::Mod => lhs.modulo(&rhs),
        }
      }
      Expression::CallExpression(call_expression) => {
        let mut args = Vec::new();
        for arg_expr in call_expression.argument_list() {
          let value = self.execute_expression(state, arg_expr)?;
          args.push(value);
        }
        Ok(self.execute_function(call_expression.name(), args)?)
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use googletest::{expect_that, gtest, prelude::*};

  use crate::{
    interpreter::{interpreter::Interpreter, value::Value},
    parser::grammar::testing::lex_and_parse_jang_file,
  };

  #[gtest]
  fn calls_main() {
    let ast = lex_and_parse_jang_file(
      r#"
        fn main() -> i32 {
          ret 1
        }
        "#
      .chars(),
    )
    .unwrap();

    let interp = Interpreter::new(ast);
    expect_that!(interp.run(), ok(pat![Value::Int32(&1)]));
  }

  #[gtest]
  fn fn_call() {
    let ast = lex_and_parse_jang_file(
      r#"
        fn other() -> i32 {
          ret 2
        }

        fn main() -> i32 {
          ret 1 + other()
        }
        "#
      .chars(),
    )
    .unwrap();

    let interp = Interpreter::new(ast);
    expect_that!(interp.run(), ok(pat![Value::Int32(&3)]));
  }

  #[gtest]
  fn locals() {
    let ast = lex_and_parse_jang_file(
      r#"
        fn add_one(x: i32) -> i32 {
          ret x + 1
        }

        fn main() -> i32 {
          ret add_one(1) + 2 * add_one(2)
        }
        "#
      .chars(),
    )
    .unwrap();

    let interp = Interpreter::new(ast);
    expect_that!(interp.run(), ok(pat![Value::Int32(&8)]));
  }

  #[gtest]
  fn integer_arithmetic() {
    let ast = lex_and_parse_jang_file(
      r#"
        fn main() -> i32 {
          ret (8 + 9 % 5) / 2 * (7 - 3) + 6 - 5
        }
        "#
      .chars(),
    )
    .unwrap();

    let interp = Interpreter::new(ast);
    expect_that!(interp.run(), ok(pat![Value::Int32(&25)]));
  }

  #[gtest]
  fn floating_point_arithmetic() {
    let ast = lex_and_parse_jang_file(
      r#"
        fn main() -> i32 {
          ret 1.1 + 1.1
        }
        "#
      .chars(),
    )
    .unwrap();

    let interp = Interpreter::new(ast);
    expect_that!(interp.run(), ok(pat![Value::Float32(&2.2)]));
  }

  #[gtest]
  fn unbound_variable_throws_error() {
    let ast = lex_and_parse_jang_file(
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
      .chars(),
    )
    .unwrap();

    let interp = Interpreter::new(ast);
    expect_that!(interp.run(), err(anything()));
  }

  #[gtest]
  fn let_bindings() {
    let ast = lex_and_parse_jang_file(
      r#"
        fn main() -> i32 {
          let x = 1
          let y = 2
          ret x + y
        }
        "#
      .chars(),
    )
    .unwrap();

    let interp = Interpreter::new(ast);
    expect_that!(interp.run(), ok(pat![Value::Int32(&3)]));
  }

  #[gtest]
  fn let_bindings_do_not() {
    let ast = lex_and_parse_jang_file(
      r#"
        fn main() -> i32 {
          let x = 1
          let y = 2
          ret x + y
        }
        "#
      .chars(),
    )
    .unwrap();

    let interp = Interpreter::new(ast);
    expect_that!(interp.run(), ok(pat![Value::Int32(&3)]));
  }
}
