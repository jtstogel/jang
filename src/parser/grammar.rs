use parser_generator::grammar;

use crate::parser::{
  ast::{
    binary_expression::{BinaryExpression, BinaryOp},
    block::{Block, BlockBuilder, NonRetBlock, RetBlock},
    expression::Expression,
    function_decl::{FunctionDecl, FunctionParameter},
    jang_file::{JangFile, JangFileBuilder},
    statement::{LetStatement, NonRetStatement, RetExpression, RetStatement},
    type_expr::Type,
  },
  token::{
    JangToken,
    ident::Ident,
    keyword::Keyword,
    operator::{Op, Operator, Spacing},
  },
};

grammar!(
  name: JangGrammar;
  enum_terminal: JangToken;

  <root>: JangFile => <jang_file> { #jang_file.build() };

  <jang_file>: JangFileBuilder => <jang_file> <function_decl> {
    #jang_file.add_function_decl(#function_decl)
  };
  <jang_file>: JangFileBuilder => ! {
    JangFileBuilder::new()
  };

  <function_decl>: FunctionDecl =>
      Keyword(Keyword::Function)
      <ident>
      <function_params>
      <function_ret_type>
      <block_scope>
  {
    FunctionDecl::new(#ident, #function_params, #function_ret_type, #block_scope)
  };

  <function_ret_type>: Option<Type> => ! { None };
  <function_ret_type>: Option<Type> => <right_arrow> <type> {
    Some(#type)
  };

  <type>: Type => <ident> { Type(#ident) };

  <block_scope>: Block => <ret_block_scope> { #ret_block_scope.into() };
  <block_scope>: Block => <non_ret_block_scope> { #non_ret_block_scope.into() };

  <ret_block_scope>: RetBlock => <open_bracket> <non_ret_statement_list> <ret_statement> <close_bracket> {
    #non_ret_statement_list.build_with_ret(#ret_statement)
  };

  <non_ret_block_scope>: NonRetBlock => <open_bracket> <non_ret_statement_list> <close_bracket> {
    #non_ret_statement_list.build()
  };

  <non_ret_statement_list>: BlockBuilder => ! { BlockBuilder::new() };
  <non_ret_statement_list>: BlockBuilder => <non_ret_statement_list> <non_ret_statement> {
    #non_ret_statement_list.with_statement(#non_ret_statement)
  };

  <non_ret_statement>: NonRetStatement => <let_binding>;
  <non_ret_statement>: NonRetStatement => <non_ret_block_scope> {
    #non_ret_block_scope.into()
  };

  <let_binding>: NonRetStatement => Keyword(Keyword::Let) <ident> <eq> <expr> {
    LetStatement::new(#ident, #expr).into()
  };

  <ret_statement>: RetStatement => <ret_block_scope> {
    #ret_block_scope.into()
  };
  <ret_statement>: RetStatement => Keyword(Keyword::Ret) <expr> {
    RetExpression::new(#expr).into()
  };

  <expr>: Expression => <add_expr>;

  <add_expr>: Expression => <add_expr> <plus> <mul_expr> {
    BinaryExpression::new(#add_expr, #mul_expr, BinaryOp::Add).into()
  };
  <add_expr>: Expression => <add_expr> <minus> <mul_expr> {
    BinaryExpression::new(#add_expr, #mul_expr, BinaryOp::Sub).into()
  };
  <add_expr>: Expression => <mul_expr>;

  <mul_expr>: Expression => <mul_expr> <mul> <leaf_expr> {
    BinaryExpression::new(#mul_expr, #leaf_expr, BinaryOp::Mul).into()
  };
  <mul_expr>: Expression => <mul_expr> <div> <leaf_expr> {
    BinaryExpression::new(#mul_expr, #leaf_expr, BinaryOp::Div).into()
  };
  <mul_expr>: Expression => <mul_expr> <modulo> <leaf_expr> {
    BinaryExpression::new(#mul_expr, #leaf_expr, BinaryOp::Mod).into()
  };
  <mul_expr>: Expression => <leaf_expr>;

  <leaf_expr>: Expression => <open_paren> <expr> <close_paren> { #expr };
  <leaf_expr>: Expression => Literal(..) {
    Expression::Literal(#0)
  };
  <leaf_expr>: Expression => Ident(..) {
    Expression::Ident(#0)
  };

  <function_params>: Vec<FunctionParameter> => <open_paren> <parameter_list> <close_paren> {
    #parameter_list
  };

  <parameter_list>: Vec<FunctionParameter> => ! { vec![] };
  <parameter_list>: Vec<FunctionParameter> => <non_empty_parameter_list>;
  <non_empty_parameter_list>: Vec<FunctionParameter> => <non_empty_parameter_list> <comma> <ident> <colon> <type> {
    #non_empty_parameter_list.push(FunctionParameter::new(#ident, #type));
    #non_empty_parameter_list
  };
  <non_empty_parameter_list>: Vec<FunctionParameter> => <ident> <colon> <type> {
    vec![
      FunctionParameter::new(#ident, #type)
    ]
  };

  <eq> => Operator(Operator { op: Op::Equal, spacing: _ });
  <open_paren> => Operator(Operator { op: Op::OpenParen, spacing: _ });
  <close_paren> => Operator(Operator { op: Op::CloseParen, spacing: _ });
  <open_bracket> => Operator(Operator { op: Op::OpenBracket, spacing: _ });
  <close_bracket> => Operator(Operator { op: Op::CloseBracket, spacing: _ });
  <plus> => Operator(Operator { op: Op::Plus, spacing: _ });
  <minus> => Operator(Operator { op: Op::Dash, spacing: _ });
  <mul> => Operator(Operator { op: Op::Star, spacing: _ });
  <div> => Operator(Operator { op: Op::Slash, spacing: _ });
  <modulo> => Operator(Operator { op: Op::Percent, spacing: _ });
  <colon> => Operator(Operator { op: Op::Colon, spacing: _ });
  <comma> => Operator(Operator { op: Op::Comma, spacing: _ });
  <right_arrow> =>
      Operator(Operator { op: Op::Dash, spacing: Spacing::Joint })
      Operator(Operator { op: Op::GreaterThan, spacing: Spacing::Alone });

  <ident>: Ident => Ident(..);
);

#[cfg(test)]
mod tests {

  use googletest::prelude::*;
  use parser_generator::parser::Parser;

  use crate::parser::{
    ast::{
      binary_expression::{BinaryOp, matchers::binary_expression as bin_exp},
      block::matchers::{block, block_with_ret, non_ret_block, ret_block},
      expression::matchers::{ident_expression as id_exp, literal_expression as lit_exp},
      function_decl::matchers::{
        fn_body, fn_name, fn_parameter_name, fn_parameter_type, fn_parameters, fn_return_type,
        fn_return_type_none,
      },
      jang_file::matchers::{jang_file_functions, jang_file_with_fn},
      statement::matchers::{let_statement as let_stmt, ret_expression as ret_expr},
      type_expr::matchers::type_expr_name,
    },
    grammar::JangGrammar,
    lexer::lex_stream,
    token::{ident::matchers::ident, literal::matchers::integral},
  };

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
      jang_file_with_fn(fn_body(block_with_ret(
        is_empty(),
        ret_expr(lit_exp(integral("123")))
      )))
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
  fn reject_two_returns() {
    let ast = JangGrammar::parse_fallible(lex_stream(
      r#"
        fn function_name() -> i32 {
          ret 123
          ret 456
        }
        "#
      .chars(),
    ));

    expect_that!(ast, err(displays_as(contains_substring("Failed to parse"))));
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
      jang_file_with_fn(fn_body(block_with_ret(
        is_empty(),
        ret_expr(id_exp(ident("x")))
      )))
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
      jang_file_with_fn(fn_body(block_with_ret(
        is_empty(),
        ret_block(is_empty(), ret_expr(id_exp(ident("x"))))
      )))
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
      jang_file_with_fn(fn_body(block_with_ret(
        elements_are![
          let_stmt(ident("x"), lit_exp(integral("123"))),
          let_stmt(ident("y"), id_exp(ident("x"))),
        ],
        ret_expr(lit_exp(integral("789")))
      )))
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
      jang_file_with_fn(fn_body(block_with_ret(
        elements_are![
          non_ret_block(elements_are![let_stmt(ident("y"), id_exp(ident("z")))]),
          let_stmt(ident("x"), lit_exp(integral("123"))),
        ],
        ret_block(
          elements_are![let_stmt(ident("x"), id_exp(ident("x")))],
          ret_block(
            is_empty(),
            ret_block(is_empty(), ret_expr(id_exp(ident("z"))))
          )
        )
      )))
    );
  }

  #[gtest]
  fn reject_let_after_ret() {
    let ast = JangGrammar::parse_fallible(lex_stream(
      r#"
        fn function_name() {
          ret 123
          let x = 456
        }
        "#
      .chars(),
    ));

    expect_that!(ast, err(displays_as(contains_substring("Failed to parse"))));
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
    let ast = JangGrammar::parse_fallible(lex_stream(
      r#"
        fn function_name() {
          let x = y + 3
        }
        "#
      .chars(),
    ))
    .unwrap();

    expect_that!(
      ast,
      jang_file_with_fn(fn_body(block(elements_are![let_stmt(
        ident("x"),
        bin_exp(id_exp(ident("y")), &BinaryOp::Add, lit_exp(integral("3")))
      )])))
    );
  }

  #[gtest]
  fn sub_expression() {
    let ast = JangGrammar::parse_fallible(lex_stream(
      r#"
        fn function_name() {
          let x = 5 - a
        }
        "#
      .chars(),
    ))
    .unwrap();

    expect_that!(
      ast,
      jang_file_with_fn(fn_body(block(elements_are![let_stmt(
        ident("x"),
        bin_exp(lit_exp(integral("5")), &BinaryOp::Sub, id_exp(ident("a")),)
      )])))
    );
  }

  #[gtest]
  fn mul_expression() {
    let ast = JangGrammar::parse_fallible(lex_stream(
      r#"
        fn function_name() {
          let x = 2 * a
        }
        "#
      .chars(),
    ))
    .unwrap();

    expect_that!(
      ast,
      jang_file_with_fn(fn_body(block(elements_are![let_stmt(
        ident("x"),
        bin_exp(lit_exp(integral("2")), &BinaryOp::Mul, id_exp(ident("a")),)
      )])))
    );
  }

  #[gtest]
  fn div_expression() {
    let ast = JangGrammar::parse_fallible(lex_stream(
      r#"
        fn function_name() {
          let x = a / b
        }
        "#
      .chars(),
    ))
    .unwrap();

    expect_that!(
      ast,
      jang_file_with_fn(fn_body(block(elements_are![let_stmt(
        ident("x"),
        bin_exp(id_exp(ident("a")), &BinaryOp::Div, id_exp(ident("b")),)
      )])))
    );
  }

  #[gtest]
  fn mod_expression() {
    let ast = JangGrammar::parse_fallible(lex_stream(
      r#"
        fn function_name() {
          let x = a % 10
        }
        "#
      .chars(),
    ))
    .unwrap();

    expect_that!(
      ast,
      jang_file_with_fn(fn_body(block(elements_are![let_stmt(
        ident("x"),
        bin_exp(id_exp(ident("a")), &BinaryOp::Mod, lit_exp(integral("10")))
      )])))
    );
  }

  #[gtest]
  fn add_left_associativity() {
    let ast = JangGrammar::parse_fallible(lex_stream(
      r#"
        fn function_name() {
          let x = a + b + c
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
          bin_exp(id_exp(ident("a")), &BinaryOp::Add, id_exp(ident("b"))),
          &BinaryOp::Add,
          id_exp(ident("c"))
        )
      )])))
    );
  }

  #[gtest]
  fn mul_left_associativity() {
    let ast = JangGrammar::parse_fallible(lex_stream(
      r#"
        fn function_name() {
          let x = a * b * c
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
          bin_exp(id_exp(ident("a")), &BinaryOp::Mul, id_exp(ident("b"))),
          &BinaryOp::Mul,
          id_exp(ident("c"))
        )
      )])))
    );
  }

  #[gtest]
  fn add_sub_equal_precedence() {
    let ast = JangGrammar::parse_fallible(lex_stream(
      r#"
        fn function_name() {
          let add_sub = a + b - c
          let sub_add = a - b + c
        }
        "#
      .chars(),
    ))
    .unwrap();

    expect_that!(
      ast,
      jang_file_with_fn(fn_body(block(elements_are![
        let_stmt(
          ident("add_sub"),
          bin_exp(
            bin_exp(id_exp(ident("a")), &BinaryOp::Add, id_exp(ident("b"))),
            &BinaryOp::Sub,
            id_exp(ident("c"))
          )
        ),
        let_stmt(
          ident("sub_add"),
          bin_exp(
            bin_exp(id_exp(ident("a")), &BinaryOp::Sub, id_exp(ident("b"))),
            &BinaryOp::Add,
            id_exp(ident("c"))
          )
        )
      ])))
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
