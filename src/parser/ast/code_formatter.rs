use std::fmt::Write;

pub struct CodeFormatter<'a, F: std::fmt::Write> {
  formatter: &'a mut F,
  spaces: Vec<u8>,
  indentation_level: usize,
  newline_queued: bool,
}

impl<'a, F: std::fmt::Write> CodeFormatter<'a, F> {
  const TAB_SPACES: usize = 2;

  pub fn new(formatter: &'a mut F) -> Self {
    Self {
      formatter,
      spaces: vec![b'\n'],
      indentation_level: 0,
      newline_queued: false,
    }
  }

  fn cur_indentation_spaces_len(&self) -> usize {
    self.indentation_level * Self::TAB_SPACES + 1
  }

  fn spaces_and_formatter(&mut self) -> (&mut F, &str) {
    let spaces =
      unsafe { str::from_utf8_unchecked(&self.spaces[..self.cur_indentation_spaces_len()]) };
    (&mut self.formatter, spaces)
  }

  fn increment_indentation(&mut self) {
    if self.spaces.len() == self.cur_indentation_spaces_len() {
      self.spaces.extend([b' ', b' ']);
    }

    self.indentation_level += 1;
  }

  fn decrement_indentation(&mut self) {
    self.indentation_level -= 1;
  }
}

impl<'a, F: std::fmt::Write> Drop for CodeFormatter<'a, F> {
  fn drop(&mut self) {
    debug_assert_eq!(self.indentation_level, 0);
    if self.newline_queued {
      self.formatter.write_str("\n").expect("Final write failed!");
    }
  }
}

impl<'a, F: std::fmt::Write> Write for CodeFormatter<'a, F> {
  fn write_str(&mut self, s: &str) -> std::fmt::Result {
    let mut newline_queued = self.newline_queued;

    for line in s.split('\n') {
      if line.is_empty() {
        newline_queued = true;
        continue;
      } else if line.ends_with('}') {
        self.decrement_indentation();
      }

      let (f, spaces) = self.spaces_and_formatter();
      if newline_queued {
        f.write_str(spaces)?;
      }

      f.write_str(line)?;

      if line.ends_with('{') {
        self.increment_indentation();
      }

      newline_queued = false;
    }

    self.newline_queued = newline_queued;

    Ok(())
  }
}

#[macro_export]
macro_rules! format_ast {
  ($($arg:tt)*) => {
    {
      let mut __string_buf = ::std::string::String::new();
      <$crate::parser::ast::code_formatter::CodeFormatter<_> as ::std::fmt::Write>::write_fmt(
        &mut $crate::parser::ast::code_formatter::CodeFormatter::new(&mut __string_buf),
        ::core::format_args!($($arg)*)
      )
      .unwrap();
      __string_buf
    }
  };
}

#[cfg(test)]
mod tests {
  use googletest::prelude::*;
  use itertools::Itertools;
  use parser_generator::parser::Parser;

  use crate::{
    error::JangResult,
    parser::{ast::jang_file::JangFile, grammar::JangGrammar, lexer::lex_stream},
  };

  fn remove_leading_whitespace(mut code: &str) -> String {
    if code.starts_with('\n') {
      code = &code[1..];
    }

    let leading_whitespace = code.chars().take_while(|ch| *ch == ' ').count();
    let prefix = format!("{:<width$}", "", width = leading_whitespace);
    code
      .lines()
      .map(|line| line.strip_prefix(&prefix).unwrap_or_default())
      .join("\n")
  }

  fn parse_code(code: &str) -> JangResult<JangFile> {
    Ok(JangGrammar::parse_fallible(lex_stream(code.chars()))?)
  }

  fn check_print_roundtrip(formatted_code: &str) -> Result<()> {
    let formatted_code = remove_leading_whitespace(formatted_code);
    let ast = parse_code(&formatted_code)?;
    let printed_code = format_ast!("{ast}");

    expect_eq!(printed_code, formatted_code);
    Ok(())
  }

  #[gtest]
  fn roundtrip_empty_fn() -> Result<()> {
    let code = r#"
      fn function_name() {
      }
    "#;

    check_print_roundtrip(code)
  }

  #[gtest]
  fn roundtrip_empty_fn_with_return_type() -> Result<()> {
    let code = r#"
      fn function_name() -> i32 {
      }
    "#;

    check_print_roundtrip(code)
  }

  #[gtest]
  fn roundtrip_fn_with_args() -> Result<()> {
    let code = r#"
      fn function_name(x: i32, y: f32) {
      }
    "#;

    check_print_roundtrip(code)
  }

  #[gtest]
  fn roundtrip_ret_fn() -> Result<()> {
    let code = r#"
      fn function_name() -> i32 {
        ret 123
      }
    "#;

    check_print_roundtrip(code)
  }

  #[gtest]
  fn roundtrip_let_stmt() -> Result<()> {
    let code = r#"
      fn function_name() {
        let x = 123
      }
    "#;

    check_print_roundtrip(code)
  }

  #[gtest]
  fn roundtrip_complex_expr() -> Result<()> {
    let code = r#"
      fn function_name() -> i32 {
        let x = x.y((3 * f())).z()
        ret (x + (x / x))
      }
    "#;

    check_print_roundtrip(code)
  }

  #[gtest]
  fn roundtrip_block_scope() -> Result<()> {
    let code = r#"
      fn function_name() -> i32 {
        {
          let y = 100
        }
        {
          {
            let z = y
          }
          {
            ret 77
          }
        }
      }
    "#;

    check_print_roundtrip(code)
  }

  #[gtest]
  fn roundtrip_if() -> Result<()> {
    let code = r#"
      fn function_name() -> i32 {
        if 1 {
        }
      }
    "#;

    check_print_roundtrip(code)
  }

  #[gtest]
  fn roundtrip_if_binary_exp() -> Result<()> {
    let code = r#"
      fn function_name() -> i32 {
        if (x + 3) {
          ret y
        }
      }
    "#;

    check_print_roundtrip(code)
  }

  #[gtest]
  fn roundtrip_if_else() -> Result<()> {
    let code = r#"
      fn function_name() -> i32 {
        if 1 {
          ret y
        } else {
          ret z
        }
      }
    "#;

    check_print_roundtrip(code)
  }

  #[gtest]
  fn roundtrip_if_else_if() -> Result<()> {
    let code = r#"
      fn function_name() -> i32 {
        if 1 {
          ret y
        } else if 2 {
          ret z
        }
        ret 0
      }
    "#;

    check_print_roundtrip(code)
  }

  #[gtest]
  fn roundtrip_structured_type() -> Result<()> {
    let code = r#"
      type X = {
        field1: i32
        field2: String
      }
    "#;

    check_print_roundtrip(code)
  }

  #[gtest]
  fn roundtrip_enum_type() -> Result<()> {
    let code = r#"
      type E = 
      | V1 
      | V2 {
        field1: i32
        field2: String
      }
      | V3 String
    "#;

    check_print_roundtrip(code)
  }
}
