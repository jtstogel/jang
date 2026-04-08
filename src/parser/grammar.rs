use parser_generator::pub_grammar;

use crate::{
  error::JangError,
  parser::{
    ast::{
      binary_expression::{BinaryExpression, BinaryOp},
      block::{Block, BlockBuilder},
      call_expression::CallExpression,
      dot_expression::DotExpression,
      enum_type_decl::{EnumTypeDecl, EnumTypeDeclBuilder, EnumVariant, EnumVariantType},
      expression::Expression,
      expression_list::{ExpressionList, ExpressionListBuilder},
      function_decl::{
        FunctionDecl, FunctionParameter, FunctionParameters, FunctionParametersBuilder,
      },
      if_statement::IfStatement,
      jang_file::{JangFile, JangFileBuilder},
      let_statement::LetStatement,
      loop_statement::LoopStatement,
      ret_statement::RetStatement,
      statement::Statement,
      structured_type_decl::{StructuredTypeDecl, StructuredTypeDeclBuilder, StructuredTypeField},
      type_decl::{TypeDecl, TypeDeclVariant},
      type_expr::TypeExpression,
    },
    token::{
      JangToken,
      ident::Ident,
      keyword::Keyword,
      literal::Literal,
      operator::{Op, Operator},
    },
  },
};

pub_grammar!(
  name: JangGrammar;
  enum_terminal: JangToken;
  error_type: JangError;

  <root>: JangFile => <jang_file>;

  <jang_file>: JangFileBuilder => <jang_file> <type_decl> {
    #jang_file.add_type_decls(#type_decl)
  };
  <jang_file>: JangFileBuilder => <jang_file> <function_decl> {
    #jang_file.add_function_decls(#function_decl)
  };
  <jang_file>: JangFileBuilder => ! {
    JangFileBuilder::default()
  };

  <type_decl>: TypeDecl =>
    Keyword(Keyword::Type) <ident> <eq> <type_decl_variant>
  {
    TypeDecl::new(#ident, #type_decl_variant)
  };

  <type_decl_variant>: TypeDeclVariant => <structured_type_decl>;
  <type_decl_variant>: TypeDeclVariant => <enum_type_decl>;

  <structured_type_decl>: StructuredTypeDecl =>
    <open_bracket> <structured_type_decl_builder> <close_bracket>
  {
    #structured_type_decl_builder.build()?
  };

  <structured_type_decl_builder>: StructuredTypeDeclBuilder => ! {
    StructuredTypeDeclBuilder::default()
  };
  <structured_type_decl_builder>: StructuredTypeDeclBuilder =>
    <structured_type_decl_builder>
    <structured_type_field>
  {
    #structured_type_decl_builder.add_fields(#structured_type_field)
  };

  <structured_type_field>: StructuredTypeField => <ident> <colon> <type_expr> {
    StructuredTypeField::new(#ident, #type_expr)
  };

  <enum_type_decl>: EnumTypeDecl => <enum_type_decl_builder>;

  <enum_type_decl_builder>: EnumTypeDeclBuilder => <enum_type_decl_variant> {
    EnumTypeDeclBuilder::default().add_variants(#enum_type_decl_variant)
  };
  <enum_type_decl_builder>: EnumTypeDeclBuilder => <enum_type_decl_builder> <enum_type_decl_variant> {
    #enum_type_decl_builder.add_variants(#enum_type_decl_variant)
  };

  <enum_type_decl_variant>: EnumVariant => <bar> <ident> <enum_variant_type> {
    EnumVariant::new(#ident, #enum_variant_type)
  };

  <enum_variant_type>: EnumVariantType => ! {
    EnumVariantType::Empty
  };
  <enum_variant_type>: EnumVariantType => <ident> {
    EnumVariantType::TypeRef(#ident)
  };
  <enum_variant_type>: EnumVariantType => <structured_type_decl> {
    EnumVariantType::Structured(#structured_type_decl)
  };

  <function_decl>: FunctionDecl =>
      Keyword(Keyword::Function)
      <ident>
      Joint
      <function_params>
      <function_ret_type>
      <block_scope>
  {
    FunctionDecl::new(#ident, #function_params, #function_ret_type, #block_scope)
  };

  <function_ret_type>: Option<TypeExpression> => ! { None };
  <function_ret_type>: Option<TypeExpression> => <right_arrow> <type_expr> {
    Some(#type_expr)
  };

  <type_expr>: TypeExpression => <ident> { TypeExpression(#ident) };

  <block_scope>: Block => <open_bracket> <statement_list> <close_bracket> {
    #statement_list
  };

  <statement_list>: Block => <statement_list_builder>;

  <statement_list_builder>: BlockBuilder => ! { BlockBuilder::default() };
  <statement_list_builder>: BlockBuilder => <statement_list_builder> <statement> {
    #statement_list_builder.add_statements(#statement)
  };

  <statement>: Statement => <let_binding>;
  <statement>: Statement => <ret_statement>;
  <statement>: Statement => <call_expr>;
  <statement>: Statement => <if_statement>;
  <statement>: Statement => <loop_statement>;
  <statement>: Statement => <block_scope>;
  <statement>: Statement => Keyword(Keyword::Break) {
    Statement::Break
  };

  <let_binding>: LetStatement => Keyword(Keyword::Let) <ident> <eq> <expr> {
    LetStatement::new(#ident, #expr)
  };

  <ret_statement>: RetStatement => Keyword(Keyword::Ret) <expr> {
    RetStatement::new(#expr)
  };

  <if_statement>: IfStatement => Keyword(Keyword::If) <expr> <block_scope> {
    IfStatement::new(#expr, #block_scope)
  };
  <if_statement>: IfStatement =>
    Keyword(Keyword::If) <expr> <block_scope>
    Keyword(Keyword::Else) <block_scope>
  {
    IfStatement::new_with_else(#expr, #2, #4)
  };
  <if_statement>: IfStatement =>
    Keyword(Keyword::If) <expr> <block_scope>
    Keyword(Keyword::Else) <if_statement>
  {
    IfStatement::new_with_else_if(#expr, #block_scope, #if_statement)
  };

  <loop_statement>: LoopStatement => Keyword(Keyword::Loop) <block_scope> {
    LoopStatement::new(#block_scope)
  };

  <expr>: Expression => <add_expr>;

  <add_expr>: Expression => <add_expr> <plus> <mul_expr> {
    BinaryExpression::new(#add_expr, #mul_expr, BinaryOp::Add).into()
  };
  <add_expr>: Expression => <add_expr> <minus> <mul_expr> {
    BinaryExpression::new(#add_expr, #mul_expr, BinaryOp::Sub).into()
  };
  <add_expr>: Expression => <mul_expr>;

  <mul_expr>: Expression => <mul_expr> <mul> <unary_expr> {
    BinaryExpression::new(#mul_expr, #unary_expr, BinaryOp::Mul).into()
  };
  <mul_expr>: Expression => <mul_expr> <div> <unary_expr> {
    BinaryExpression::new(#mul_expr, #unary_expr, BinaryOp::Div).into()
  };
  <mul_expr>: Expression => <mul_expr> <modulo> <unary_expr> {
    BinaryExpression::new(#mul_expr, #unary_expr, BinaryOp::Mod).into()
  };
  <mul_expr>: Expression => <unary_expr>;

  // Unary expressions:
  <unary_expr>: Expression => <call_or_dot_expr>;
  <unary_expr>: Expression => <literal>;

  // Call expressions.
  <call_or_dot_expr>: Expression => <call_expr>;
  <call_or_dot_expr>: Expression => <call_or_dot_expr> <dot> <ident> {
    DotExpression::new(#call_or_dot_expr, #ident).into()
  };
  <call_or_dot_expr>: Expression => <leaf_expr>;

  <call_expr>: CallExpression => <call_or_dot_expr> Joint <call_args> {
    CallExpression::new(#call_or_dot_expr, #call_args)
  };

  <call_args>: ExpressionList => <open_paren> <expr_list_builder> <close_paren> {
    #expr_list_builder.build()?
  };

  <leaf_expr>: Expression => <open_paren> <expr> <close_paren> { #expr };
  <leaf_expr>: Expression => <ident>;

  <expr_list_builder>: ExpressionListBuilder => ! { ExpressionListBuilder::default() };
  <expr_list_builder>: ExpressionListBuilder => <non_empty_expr_list>;

  <non_empty_expr_list>: ExpressionListBuilder => <expr> {
    ExpressionListBuilder::default().add_expressions(#expr)
  };
  <non_empty_expr_list>: ExpressionListBuilder => <non_empty_expr_list> <comma> <expr> {
    #non_empty_expr_list.add_expressions(#expr)
  };

  <function_params>: FunctionParameters => <open_paren> <parameter_list> <close_paren> {
    #parameter_list.build()?
  };

  <parameter_list>: FunctionParametersBuilder => ! { FunctionParametersBuilder::default() };
  <parameter_list>: FunctionParametersBuilder => <non_empty_parameter_list>;
  <non_empty_parameter_list>: FunctionParametersBuilder =>
      <non_empty_parameter_list> <comma> <ident> <colon> <type_expr>
  {
    #non_empty_parameter_list.add_parameters(FunctionParameter::new(#ident, #type_expr))
  };
  <non_empty_parameter_list>: FunctionParametersBuilder => <ident> <colon> <type_expr> {
    let builder = FunctionParametersBuilder::default();
    builder.add_parameters(FunctionParameter::new(#ident, #type_expr))
  };

  <eq> => Operator(Operator { op: Op::Equal });
  <open_paren> => Operator(Operator { op: Op::OpenParen });
  <close_paren> => Operator(Operator { op: Op::CloseParen });
  <open_bracket> => Operator(Operator { op: Op::OpenBracket });
  <close_bracket> => Operator(Operator { op: Op::CloseBracket });
  <plus> => Operator(Operator { op: Op::Plus });
  <minus> => Operator(Operator { op: Op::Dash });
  <mul> => Operator(Operator { op: Op::Star });
  <div> => Operator(Operator { op: Op::Slash });
  <modulo> => Operator(Operator { op: Op::Percent });
  <colon> => Operator(Operator { op: Op::Colon });
  <comma> => Operator(Operator { op: Op::Comma });
  <dot> => Operator(Operator { op: Op::Dot });
  <bar> => Operator(Operator { op: Op::Bar });
  <right_arrow> =>
      Operator(Operator { op: Op::Dash })
      Joint
      Operator(Operator { op: Op::GreaterThan });

  <literal>: Literal => Literal(..);
  <ident>: Ident => Ident(..);
);

#[cfg(test)]
pub mod testing {
  use parser_generator::parser::Parser;

  use crate::{
    error::JangResult,
    parser::{ast::jang_file::JangFile, grammar::JangGrammar, lexer::lex_stream},
  };

  pub fn lex_and_parse_jang_file(text: impl IntoIterator<Item = char>) -> JangResult<JangFile> {
    Ok(JangGrammar::parse_fallible(lex_stream(text))?)
  }
}

#[cfg(test)]
mod tests {
  use std::{error::Error, fmt::Debug};

  use googletest::prelude::*;
  use parser_generator::parser::Parser;

  use crate::{
    error::JangResult,
    parser::{
      ast::{
        binary_expression::{BinaryOp, matchers::binary_expression as bin_exp},
        block::matchers::{block, block_statement},
        call_expression::matchers::{
          call_expr_args, call_expr_target, call_expression, call_statement,
        },
        dot_expression::matchers::{dot_expr_base, dot_expr_member},
        enum_type_decl::matchers::{
          enum_ref_type, enum_structured_type, enum_variant, enum_variant_with,
        },
        expression::{
          Expression,
          matchers::{ident_expression as id_exp, literal_expression as lit_exp},
        },
        function_decl::matchers::{
          fn_body, fn_name, fn_parameter_name, fn_parameter_type, fn_parameters, fn_return_type,
          fn_return_type_none,
        },
        if_statement::matchers::{
          if_else_clause, if_else_if_statement, if_else_statement, if_statement,
        },
        jang_file::matchers::{jang_file_functions, jang_file_with_fn, jang_file_with_type},
        let_statement::matchers::let_statement as let_stmt,
        loop_statement::matchers::loop_statement,
        ret_statement::matchers::ret_statement as ret_stmt,
        statement::{Statement, matchers::break_statement},
        structured_type_decl::matchers::type_field,
        type_decl::matchers::{enum_type, structured_type},
        type_expr::matchers::type_expr_name,
      },
      grammar::JangGrammar,
      lexer::lex_stream,
      token::{ident::matchers::ident, literal::matchers::integral},
    },
  };

  fn failed_to_parse<'a, T: Debug, E: Error>() -> impl Matcher<&'a std::result::Result<T, E>> {
    err(displays_as(contains_substring("Failed to parse")))
  }

  fn parse_single_exp(expression: &str) -> JangResult<Expression> {
    let ast = JangGrammar::parse_fallible(lex_stream(
      format!(
        r#"
        fn function_name() {{
          let x = {}
        }}
        "#,
        expression
      )
      .chars(),
    ))?;

    let statement = &ast.function_decls()[0].body().statements()[0];
    match statement {
      Statement::Let(stmt) => Ok(stmt.expr().clone()),
      _ => {
        panic!(
          "parse_single_exp expects a let statement, got: {}",
          statement
        )
      }
    }
  }

  #[gtest]
  fn parse_minimal_function() {
    let ast = JangGrammar::parse_fallible(lex_stream(
      r#"
        fn function_name() -> i32 {
          ret 123
        }
        "#
      .chars(),
    ))
    .unwrap();

    expect_that!(ast, jang_file_with_fn(fn_name(ident("function_name"))));
    expect_that!(
      ast,
      jang_file_with_fn(fn_return_type(type_expr_name(ident("i32"))))
    );
    expect_that!(
      ast,
      jang_file_with_fn(fn_body(block(elements_are![ret_stmt(lit_exp(integral(
        "123"
      )))])))
    );
    expect_that!(ast, jang_file_with_fn(fn_parameters(is_empty())));
  }

  #[gtest]
  fn function_without_return_type() {
    let ast = JangGrammar::parse_fallible(lex_stream(
      r#"
        fn function_name() {}
        "#
      .chars(),
    ))
    .unwrap();

    expect_that!(ast, jang_file_with_fn(fn_name(ident("function_name"))));
    expect_that!(ast, jang_file_with_fn(fn_return_type_none()));
    expect_that!(ast, jang_file_with_fn(fn_body(block(is_empty()))));
    expect_that!(ast, jang_file_with_fn(fn_parameters(is_empty())));
  }

  #[gtest]
  fn empty_type_decl() {
    let ast = JangGrammar::parse_fallible(lex_stream(
      r#"
        type X = {}
        "#
      .chars(),
    ))
    .unwrap();

    expect_that!(
      ast,
      jang_file_with_type(structured_type(ident("X"), is_empty()))
    );
  }

  #[gtest]
  fn type_decl_one_field() {
    let ast = JangGrammar::parse_fallible(lex_stream(
      r#"
        type X = {
          field1: i32
        }
        "#
      .chars(),
    ))
    .unwrap();

    expect_that!(
      ast,
      jang_file_with_type(structured_type(
        ident("X"),
        elements_are![type_field(ident("field1"), type_expr_name(ident("i32")))]
      ))
    );
  }

  #[gtest]
  fn type_decl_many_fields() {
    let ast = JangGrammar::parse_fallible(lex_stream(
      r#"
        type X = {
          a: i32
          b: String
          c: MyCustomType
        }
        "#
      .chars(),
    ))
    .unwrap();

    expect_that!(
      ast,
      jang_file_with_type(structured_type(
        ident("X"),
        unordered_elements_are![
          type_field(ident("a"), type_expr_name(ident("i32"))),
          type_field(ident("b"), type_expr_name(ident("String"))),
          type_field(ident("c"), type_expr_name(ident("MyCustomType"))),
        ]
      ))
    );
  }

  #[gtest]
  fn enum_decl_single_variant() {
    let ast = JangGrammar::parse_fallible(lex_stream(
      r#"
        type E =
          | V1
        "#
      .chars(),
    ))
    .unwrap();

    expect_that!(
      ast,
      jang_file_with_type(enum_type(
        ident("E"),
        elements_are![enum_variant(ident("V1"))]
      ))
    );
  }

  #[gtest]
  fn enum_decl_single_variant_with_data() {
    let ast = JangGrammar::parse_fallible(lex_stream(
      r#"
        type E =
          | V1 i32
        "#
      .chars(),
    ))
    .unwrap();

    expect_that!(
      ast,
      jang_file_with_type(enum_type(
        ident("E"),
        elements_are![enum_variant_with(ident("V1"), enum_ref_type(ident("i32")))]
      ))
    );
  }

  #[gtest]
  fn enum_decl_structured_variant() {
    let ast = JangGrammar::parse_fallible(lex_stream(
      r#"
        type E =
          | V1 { field1: i32 }
        "#
      .chars(),
    ))
    .unwrap();

    expect_that!(
      ast,
      jang_file_with_type(enum_type(
        ident("E"),
        elements_are![enum_variant_with(
          ident("V1"),
          enum_structured_type(elements_are![type_field(
            ident("field1"),
            type_expr_name(ident("i32"))
          )])
        )]
      ))
    );
  }

  #[gtest]
  fn enum_decl_many_variants() {
    let ast = JangGrammar::parse_fallible(lex_stream(
      r#"
        type E =
          | V1
          | V2 {
              f1: i64
              f2: String
            }
          | V3 String
        "#
      .chars(),
    ))
    .unwrap();

    expect_that!(
      ast,
      jang_file_with_type(enum_type(
        ident("E"),
        elements_are![
          enum_variant(ident("V1")),
          enum_variant_with(
            ident("V2"),
            enum_structured_type(elements_are![
              type_field(ident("f1"), type_expr_name(ident("i64"))),
              type_field(ident("f2"), type_expr_name(ident("String"))),
            ])
          ),
          enum_variant_with(ident("V3"), enum_ref_type(ident("String"))),
        ]
      ))
    );
  }

  #[gtest]
  fn ret_ident() {
    let ast = JangGrammar::parse_fallible(lex_stream(
      r#"
        fn function_name() -> i32 {
          ret x
        }
        "#
      .chars(),
    ))
    .unwrap();

    expect_that!(
      ast,
      jang_file_with_fn(fn_body(block(elements_are![ret_stmt(id_exp(ident("x")))])))
    );
  }

  #[gtest]
  fn ret_in_block() {
    let ast = JangGrammar::parse_fallible(lex_stream(
      r#"
        fn function_name() -> i32 {
          {
            ret x
          }
        }
        "#
      .chars(),
    ))
    .unwrap();

    expect_that!(
      ast,
      jang_file_with_fn(fn_body(block(elements_are![block_statement(
        elements_are![ret_stmt(id_exp(ident("x")))]
      )])))
    );
  }

  #[gtest]
  fn let_binding() {
    let ast = JangGrammar::parse_fallible(lex_stream(
      r#"
        fn function_name() {
          let x = 123
        }
        "#
      .chars(),
    ))
    .unwrap();

    expect_that!(
      ast,
      jang_file_with_fn(fn_body(block(elements_are![let_stmt(
        ident("x"),
        lit_exp(integral("123"))
      )])))
    );
  }

  #[gtest]
  fn lets_followed_by_ret() {
    let ast = JangGrammar::parse_fallible(lex_stream(
      r#"
        fn function_name() {
          let x = 123
          let y = x
          ret 789
        }
        "#
      .chars(),
    ))
    .unwrap();

    expect_that!(
      ast,
      jang_file_with_fn(fn_body(block(elements_are![
        let_stmt(ident("x"), lit_exp(integral("123"))),
        let_stmt(ident("y"), id_exp(ident("x"))),
        ret_stmt(lit_exp(integral("789")))
      ])))
    );
  }

  #[gtest]
  fn call_expr_statement() {
    let ast = JangGrammar::parse_fallible(lex_stream(
      r#"
        fn function_name() {
          f()
          x(1 + y)
          (x + 1)(y)
        }
        "#
      .chars(),
    ))
    .unwrap();

    expect_that!(
      ast,
      jang_file_with_fn(fn_body(block(elements_are![
        call_statement(all![
          call_expr_target(id_exp(ident("f"))),
          call_expr_args(is_empty())
        ]),
        call_statement(all![
          call_expr_target(id_exp(ident("x"))),
          call_expr_args(elements_are![bin_exp(
            lit_exp(integral("1")),
            &BinaryOp::Add,
            id_exp(ident("y"))
          )])
        ]),
        call_statement(all![
          call_expr_target(bin_exp(
            id_exp(ident("x")),
            &BinaryOp::Add,
            lit_exp(integral("1"))
          )),
          call_expr_args(elements_are![id_exp(ident("y"))])
        ])
      ])))
    );
  }

  #[gtest]
  fn nested_blocks() {
    let ast = JangGrammar::parse_fallible(lex_stream(
      r#"
        fn function_name() {
          {
            let y = z
          }
          let x = 123
          {
            let x = x
            {
              {
                ret z
              }
            }
          }
        }
        "#
      .chars(),
    ))
    .unwrap();

    expect_that!(
      ast,
      jang_file_with_fn(fn_body(block(elements_are![
        block_statement(elements_are![let_stmt(ident("y"), id_exp(ident("z")))]),
        let_stmt(ident("x"), lit_exp(integral("123"))),
        block_statement(elements_are![
          let_stmt(ident("x"), id_exp(ident("x"))),
          block_statement(elements_are![block_statement(elements_are![ret_stmt(
            id_exp(ident("z"))
          )])])
        ])
      ])))
    );
  }

  #[gtest]
  fn parse_unary_function() {
    let ast = JangGrammar::parse_fallible(lex_stream(
      r#"
        fn func(a: i32) {
          ret 123
        }
        "#
      .chars(),
    ))
    .unwrap();

    expect_that!(
      ast,
      jang_file_with_fn(fn_parameters(elements_are![all!(
        fn_parameter_name(ident("a")),
        fn_parameter_type(type_expr_name(ident("i32")))
      )]))
    );
  }

  #[gtest]
  fn parse_multi_param_function() {
    let ast = JangGrammar::parse_fallible(lex_stream(
      r#"
        fn func(a: i32, b: i32) {
          ret 123
        }
        "#
      .chars(),
    ))
    .unwrap();

    expect_that!(
      ast,
      jang_file_with_fn(fn_parameters(elements_are![
        all!(fn_parameter_name(ident("a"))),
        all!(fn_parameter_name(ident("b")))
      ]))
    );
  }

  #[gtest]
  fn add_expression() {
    let expr = "y + 3";
    expect_that!(
      parse_single_exp(expr).unwrap(),
      bin_exp(id_exp(ident("y")), &BinaryOp::Add, lit_exp(integral("3")))
    );
  }

  #[gtest]
  fn sub_expression() {
    let expr = "5 - a";
    expect_that!(
      parse_single_exp(expr).unwrap(),
      bin_exp(lit_exp(integral("5")), &BinaryOp::Sub, id_exp(ident("a"))),
    );
  }

  #[gtest]
  fn mul_expression() {
    let expr = "2 * a";
    expect_that!(
      parse_single_exp(expr).unwrap(),
      bin_exp(lit_exp(integral("2")), &BinaryOp::Mul, id_exp(ident("a")),)
    );
  }

  #[gtest]
  fn div_expression() {
    let expression = "a / b";
    expect_that!(
      parse_single_exp(expression).unwrap(),
      bin_exp(id_exp(ident("a")), &BinaryOp::Div, id_exp(ident("b")))
    );
  }

  #[gtest]
  fn mod_expression() {
    let expr = "a % 10";
    expect_that!(
      parse_single_exp(expr).unwrap(),
      bin_exp(id_exp(ident("a")), &BinaryOp::Mod, lit_exp(integral("10")))
    );
  }

  #[gtest]
  fn add_left_associativity() {
    let expr = "a + b + c";
    expect_that!(
      parse_single_exp(expr).unwrap(),
      bin_exp(
        bin_exp(id_exp(ident("a")), &BinaryOp::Add, id_exp(ident("b"))),
        &BinaryOp::Add,
        id_exp(ident("c"))
      )
    );
  }

  #[gtest]
  fn mul_left_associativity() {
    let expr = "a * b * c";
    expect_that!(
      parse_single_exp(expr).unwrap(),
      bin_exp(
        bin_exp(id_exp(ident("a")), &BinaryOp::Mul, id_exp(ident("b"))),
        &BinaryOp::Mul,
        id_exp(ident("c"))
      )
    );
  }

  #[gtest]
  fn add_sub_equal_precedence() {
    let expr = "a + b - c";
    expect_that!(
      parse_single_exp(expr).unwrap(),
      bin_exp(
        bin_exp(id_exp(ident("a")), &BinaryOp::Add, id_exp(ident("b"))),
        &BinaryOp::Sub,
        id_exp(ident("c"))
      )
    );

    let expr = "a - b + c";
    expect_that!(
      parse_single_exp(expr).unwrap(),
      bin_exp(
        bin_exp(id_exp(ident("a")), &BinaryOp::Sub, id_exp(ident("b"))),
        &BinaryOp::Add,
        id_exp(ident("c"))
      )
    );
  }

  #[gtest]
  fn mul_higher_precedence() {
    let expr = "a + b * c - d / e";
    expect_that!(
      parse_single_exp(expr).unwrap(),
      bin_exp(
        bin_exp(
          id_exp(ident("a")),
          &BinaryOp::Add,
          bin_exp(id_exp(ident("b")), &BinaryOp::Mul, id_exp(ident("c")))
        ),
        &BinaryOp::Sub,
        bin_exp(id_exp(ident("d")), &BinaryOp::Div, id_exp(ident("e")))
      )
    );
  }

  #[gtest]
  fn mul_div_mod_equal_precedence() {
    let ast = JangGrammar::parse_fallible(lex_stream(
      r#"
        fn function_name() {
          let mdm = a * b / c % d
          let dmm = a / b % c * d
          let mmd = a % b * c / d
        }
        "#
      .chars(),
    ))
    .unwrap();

    expect_that!(
      ast,
      jang_file_with_fn(fn_body(block(elements_are![
        let_stmt(
          ident("mdm"),
          bin_exp(
            bin_exp(
              bin_exp(id_exp(ident("a")), &BinaryOp::Mul, id_exp(ident("b"))),
              &BinaryOp::Div,
              id_exp(ident("c"))
            ),
            &BinaryOp::Mod,
            id_exp(ident("d"))
          )
        ),
        let_stmt(
          ident("dmm"),
          bin_exp(
            bin_exp(
              bin_exp(id_exp(ident("a")), &BinaryOp::Div, id_exp(ident("b"))),
              &BinaryOp::Mod,
              id_exp(ident("c"))
            ),
            &BinaryOp::Mul,
            id_exp(ident("d"))
          )
        ),
        let_stmt(
          ident("mmd"),
          bin_exp(
            bin_exp(
              bin_exp(id_exp(ident("a")), &BinaryOp::Mod, id_exp(ident("b"))),
              &BinaryOp::Mul,
              id_exp(ident("c"))
            ),
            &BinaryOp::Div,
            id_exp(ident("d"))
          )
        )
      ])))
    );
  }

  #[gtest]
  fn call_expr() {
    let expr = "y() + z()";
    expect_that!(
      parse_single_exp(expr).unwrap(),
      bin_exp(
        call_expression(all![
          call_expr_target(id_exp(ident("y"))),
          call_expr_args(is_empty())
        ]),
        &BinaryOp::Add,
        call_expression(all![
          call_expr_target(id_exp(ident("z"))),
          call_expr_args(is_empty())
        ]),
      )
    );
  }

  #[gtest]
  fn call_expr_with_args() {
    let expr = "y(1) + z(w, 2 + 3)";
    expect_that!(
      parse_single_exp(expr).unwrap(),
      bin_exp(
        call_expression(all![
          call_expr_target(id_exp(ident("y"))),
          call_expr_args(elements_are![lit_exp(integral("1"))])
        ]),
        &BinaryOp::Add,
        call_expression(all![
          call_expr_target(id_exp(ident("z"))),
          call_expr_args(elements_are![
            id_exp(ident("w")),
            bin_exp(
              lit_exp(integral("2")),
              &BinaryOp::Add,
              lit_exp(integral("3"))
            )
          ])
        ]),
      )
    );
  }

  #[gtest]
  fn dot_expr() {
    let expr = "y.z";
    expect_that!(
      parse_single_exp(expr).unwrap(),
      all![
        dot_expr_base(id_exp(ident("y"))),
        dot_expr_member(ident("z"))
      ]
    );
  }

  #[gtest]
  fn call_dot_expr() {
    let expr = "y.z()";
    expect_that!(
      parse_single_exp(expr).unwrap(),
      call_expression(all![
        call_expr_target(all![
          dot_expr_base(id_exp(ident("y"))),
          dot_expr_member(ident("z"))
        ]),
        call_expr_args(is_empty())
      ])
    );
  }

  #[gtest]
  fn call_paren_expr() {
    let expr = "(y.z)()";
    expect_that!(
      parse_single_exp(expr).unwrap(),
      call_expression(all![
        call_expr_target(all![
          dot_expr_base(id_exp(ident("y"))),
          dot_expr_member(ident("z"))
        ]),
        call_expr_args(is_empty())
      ])
    );
  }

  #[gtest]
  fn dot_paren_expr() {
    let expr = "(x + 3).y";
    expect_that!(
      parse_single_exp(expr).unwrap(),
      all![
        dot_expr_base(bin_exp(
          id_exp(ident("x")),
          &BinaryOp::Add,
          lit_exp(integral("3"))
        )),
        dot_expr_member(ident("y"))
      ]
    );
  }

  #[gtest]
  fn chained_dot_expr() {
    let expr = "a.b.c().d";
    expect_that!(
      parse_single_exp(expr).unwrap(),
      all![
        dot_expr_base(call_expression(all![
          call_expr_target(all![
            dot_expr_base(all![
              dot_expr_base(id_exp(ident("a"))),
              dot_expr_member(ident("b"))
            ]),
            dot_expr_member(ident("c"))
          ]),
          call_expr_args(is_empty())
        ])),
        dot_expr_member(ident("d"))
      ]
    );
  }

  #[gtest]
  fn cannot_call_literal() {
    let ast = JangGrammar::parse_fallible(lex_stream(
      r#"
        fn function_name() {
          let x = 1()
        }
        "#
      .chars(),
    ));

    expect_that!(ast, failed_to_parse());
  }

  #[gtest]
  fn standalone_if_statement() {
    let ast = JangGrammar::parse_fallible(lex_stream(
      r#"
        fn function_name() {
          if x {}
        }
        "#
      .chars(),
    ))
    .unwrap();

    expect_that!(
      ast,
      jang_file_with_fn(fn_body(block(elements_are![if_statement(
        id_exp(ident("x")),
        block(is_empty())
      )])))
    );
  }

  #[gtest]
  fn if_statement_with_body() {
    let ast = JangGrammar::parse_fallible(lex_stream(
      r#"
        fn function_name() {
          if x + 3 {
            let y = 1
            ret z
          }
        }
        "#
      .chars(),
    ))
    .unwrap();

    expect_that!(
      ast,
      jang_file_with_fn(fn_body(block(elements_are![if_statement(
        bin_exp(id_exp(ident("x")), &BinaryOp::Add, lit_exp(integral("3"))),
        block(elements_are![
          let_stmt(ident("y"), lit_exp(integral("1"))),
          ret_stmt(id_exp(ident("z")))
        ])
      )])))
    );
  }

  #[gtest]
  fn if_else() {
    let ast = JangGrammar::parse_fallible(lex_stream(
      r#"
        fn function_name() {
          if y(2) {
            ret z
          } else {
            ret 10
          }
        }
        "#
      .chars(),
    ))
    .unwrap();

    expect_that!(
      ast,
      jang_file_with_fn(fn_body(block(elements_are![if_else_statement(
        call_expression(all![
          call_expr_target(id_exp(ident("y"))),
          call_expr_args(elements_are![lit_exp(integral("2"))])
        ]),
        block(elements_are![ret_stmt(id_exp(ident("z")))]),
        block(elements_are![ret_stmt(lit_exp(integral("10")))])
      )])))
    );
  }

  #[gtest]
  fn if_else_if() {
    let ast = JangGrammar::parse_fallible(lex_stream(
      r#"
        fn function_name() {
          if x {
            ret 1
          } else if y {
            ret 2
          } else {
            let a = 3
            ret a
          }
        }
        "#
      .chars(),
    ))
    .unwrap();

    expect_that!(
      ast,
      jang_file_with_fn(fn_body(block(elements_are![if_else_if_statement(
        id_exp(ident("x")),
        block(elements_are![ret_stmt(lit_exp(integral("1")))]),
        if_else_clause(
          id_exp(ident("y")),
          block(elements_are![ret_stmt(lit_exp(integral("2")))]),
          block(elements_are![
            let_stmt(ident("a"), lit_exp(integral("3"))),
            ret_stmt(id_exp(ident("a")))
          ])
        )
      )])))
    );
  }

  #[gtest]
  fn empty_loop() {
    let ast = JangGrammar::parse_fallible(lex_stream(
      r#"
        fn function_name() {
          loop {
          }
        }
        "#
      .chars(),
    ))
    .unwrap();

    expect_that!(
      ast,
      jang_file_with_fn(fn_body(block(elements_are![loop_statement(is_empty())])))
    );
  }

  #[gtest]
  fn loop_with_statements() {
    let ast = JangGrammar::parse_fallible(lex_stream(
      r#"
        fn function_name() {
          loop {
            let x = 3
            super_fn()
          }
        }
        "#
      .chars(),
    ))
    .unwrap();

    expect_that!(
      ast,
      jang_file_with_fn(fn_body(block(elements_are![loop_statement(
        elements_are![
          let_stmt(ident("x"), lit_exp(integral("3"))),
          call_statement(call_expr_target(id_exp(ident("super_fn"))))
        ]
      )])))
    );
  }

  #[gtest]
  fn loop_with_break() {
    let ast = JangGrammar::parse_fallible(lex_stream(
      r#"
        fn function_name() {
          loop {
            break
          }
        }
        "#
      .chars(),
    ))
    .unwrap();

    expect_that!(
      ast,
      jang_file_with_fn(fn_body(block(elements_are![loop_statement(
        elements_are![break_statement()]
      )])))
    );
  }

  #[gtest]
  fn parenthesis_expression() {
    let ast = JangGrammar::parse_fallible(lex_stream(
      r#"
        fn function_name() {
          let x = (y + z) - (w * 3)
        }
        "#
      .chars(),
    ))
    .unwrap();

    expect_that!(
      ast,
      jang_file_with_fn(fn_body(block(elements_are![let_stmt(
        ident("x"),
        bin_exp(
          bin_exp(id_exp(ident("y")), &BinaryOp::Add, id_exp(ident("z"))),
          &BinaryOp::Sub,
          bin_exp(id_exp(ident("w")), &BinaryOp::Mul, lit_exp(integral("3")))
        )
      )])))
    );
  }

  #[gtest]
  fn many_parenthesis() {
    let ast = JangGrammar::parse_fallible(lex_stream(
      r#"
        fn function_name() {
          let x = (((((((((y))))) + (((((((z)))))))))))
        }
        "#
      .chars(),
    ))
    .unwrap();

    expect_that!(
      ast,
      jang_file_with_fn(fn_body(block(elements_are![let_stmt(
        ident("x"),
        bin_exp(id_exp(ident("y")), &BinaryOp::Add, id_exp(ident("z")))
      )])))
    );
  }

  #[gtest]
  fn empty_jang_file() {
    let ast = JangGrammar::parse_fallible(lex_stream("".chars())).unwrap();

    expect_that!(ast, jang_file_functions(is_empty()));
  }

  #[gtest]
  fn multiple_function_jang_file() {
    let ast = JangGrammar::parse_fallible(lex_stream(
      r#"
        fn a() {}

        fn b() {}

        fn c() {}
        "#
      .chars(),
    ))
    .unwrap();

    expect_that!(
      ast,
      jang_file_functions(elements_are![
        fn_name(ident("a")),
        fn_name(ident("b")),
        fn_name(ident("c"))
      ])
    );
  }
}
