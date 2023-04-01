use std::{
  collections::HashMap,
  io::{BufRead, BufReader},
};

use crate::{
  compiler::Compiler, data_struct::IError, interpreter::Interpreter,
  utils::convert_macro_robson, CompilerInfra, Infra,
};

pub struct TestInfra {
  stdin: String,
  stdout: String,
}
impl TestInfra {
  fn new(stdin: String) -> Self {
    Self {
      stdin,
      stdout: String::new(),
    }
  }
}
impl CompilerInfra for TestInfra {
  fn println(&mut self, to_print: String) {
    self.stdout.push_str(&format!("{}\n", to_print))
  }

  fn clone_self(&mut self) -> Box<dyn CompilerInfra> {
    Box::new(TestInfra {
      stdin: String::new(),
      stdout: String::new(),
    })
  }
  fn color_print(&mut self, _: String, _: u64) {}

  fn home_dir(&self) -> Option<String> {
    Some(String::from("/"))
  }
  fn lines(&self, path: &str) -> Result<Vec<String>, IError> {
    let file = std::fs::File::options().read(true).open(path)?;
    let buff_reader = BufReader::new(&file);
    let lines = buff_reader
      .lines()
      .flat_map(|a| a.ok())
      .collect::<Vec<String>>();
    Ok(lines)
  }
}

impl Infra for TestInfra {
  fn is_raw_mode(&self) -> bool {
    false
  }
  fn print(&mut self, to_print: &[u8]) {
    self.stdout.push_str(&String::from_utf8_lossy(&to_print));
  }
  fn println(&mut self, to_print: String) {
    self.stdout.push_str(&format!("{}\n", to_print))
  }
  fn read_line(&mut self) -> Result<String, std::io::Error> {
    let input = self.stdin.clone();
    let split: Vec<&str> = input.split('\n').collect();
    self.stdin = split[1..split.len()]
      .iter()
      .map(|a| format!("{}\n", a))
      .collect();
    Ok(split[0].to_owned())
  }
  fn clear_all(&mut self) -> Result<(), crate::data_struct::IError> {
    Ok(())
  }
  fn clear_purge(
    &mut self,
  ) -> Result<(), crate::data_struct::IError> {
    Ok(())
  }
  fn disable_raw_mode(
    &mut self,
  ) -> Result<(), crate::data_struct::IError> {
    Ok(())
  }
  fn enable_raw_mode(
    &mut self,
  ) -> Result<(), crate::data_struct::IError> {
    Ok(())
  }
  fn flush(&mut self) {}
  fn hide_cursor(
    &mut self,
  ) -> Result<(), crate::data_struct::IError> {
    Ok(())
  }
  fn move_cursor(
    &mut self,
    _x: u32,
    _y: u32,
  ) -> Result<(), crate::data_struct::IError> {
    Ok(())
  }
  fn poll(
    &self,
    _duration: u64,
  ) -> Result<u32, crate::data_struct::IError> {
    Ok(0)
  }
  fn show_cursor(
    &mut self,
  ) -> Result<(), crate::data_struct::IError> {
    Ok(())
  }
  fn use_color(
    &mut self,
    _: u32,
  ) -> Result<(), crate::data_struct::IError> {
    Ok(())
  }
  fn use_background(
    &mut self,
    _color: u32,
  ) -> Result<(), crate::data_struct::IError> {
    Ok(())
  }
}

#[test]
fn push_and_print() {
  let mut compiler = Compiler::new(
    "tests/push.robson".to_owned(),
    Box::new(TestInfra::new("".to_owned())),
  )
  .unwrap();
  let compiled = compiler.compile().unwrap();
  let mut infra = TestInfra::new("12\ntesteteste123".to_owned());

  let mut interpreter = Interpreter::<10>::new(&compiled);
  interpreter.run_buffer(&mut infra).unwrap()
}
#[test]
fn jump() {
  let mut compiler = Compiler::new(
    "tests/jump.robson".to_owned(),
    Box::new(TestInfra::new("".to_owned())),
  )
  .unwrap();
  let compiled = compiler.compile().unwrap();
  let mut infra = TestInfra::new("12\ntesteteste123".to_owned());

  let mut interpreter = Interpreter::<10>::new(&compiled);
  interpreter.run_buffer(&mut infra).unwrap()
}

#[test]
fn memory() {
  let mut compiler = Compiler::new(
    "tests/memory.robson".to_owned(),
    Box::new(TestInfra::new("".to_owned())),
  )
  .unwrap();
  let mut infra = TestInfra::new("12\ntesteteste123".to_owned());

  let compiled = compiler.compile().unwrap();
  let mut interpreter = Interpreter::<10>::new(&compiled);
  interpreter.run_buffer(&mut infra).unwrap()
}

#[test]
fn if_() {
  let mut compiler = Compiler::new(
    "tests/if.robson".to_owned(),
    Box::new(TestInfra::new("".to_owned())),
  )
  .unwrap();
  let compiled = compiler.compile().unwrap();
  let mut infra = TestInfra::new("12\ntesteteste123".to_owned());

  let mut interpreter = Interpreter::<10>::new(&compiled);
  interpreter.run_buffer(&mut infra).unwrap()
}
#[test]
fn input() {
  let mut compiler = Compiler::new(
    "tests/input.robson".to_owned(),
    Box::new(TestInfra::new("".to_owned())),
  )
  .unwrap();
  let mut infra = TestInfra::new("12\ntesteteste123".to_owned());
  let compiled = compiler.compile().unwrap();
  let mut interpreter = Interpreter::<10>::new(&compiled);
  interpreter.run_buffer(&mut infra).unwrap()
}
#[test]
fn operations() {
  let mut compiler = Compiler::new(
    "tests/operations.robson".to_owned(),
    Box::new(TestInfra::new("".to_owned())),
  )
  .unwrap();
  let compiled = compiler.compile().unwrap();

  let mut infra = TestInfra::new("12\ntesteteste123".to_owned());

  let mut interpreter = Interpreter::<10>::new(&compiled);
  interpreter.run_buffer(&mut infra).unwrap()
}

#[test]
fn types() {
  let mut compiler = Compiler::new(
    "tests/types.robson".to_owned(),
    Box::new(TestInfra::new("".to_owned())),
  )
  .unwrap();
  let mut infra = TestInfra::new("12\ntesteteste123".to_owned());

  let compiled = compiler.compile().unwrap();
  let mut interpreter = Interpreter::<10>::new(&compiled);
  interpreter.run_buffer(&mut infra).unwrap()
}

#[test]
fn include() {
  let mut compiler = Compiler::new(
    "tests/include.robson".to_owned(),
    Box::new(TestInfra::new("".to_owned())),
  )
  .unwrap();
  let compiled = compiler.compile().unwrap();
  let mut infra = TestInfra::new("12\ntesteteste123".to_owned());

  let mut interpreter = Interpreter::<10>::new(&compiled);
  interpreter.run_buffer(&mut infra).unwrap()
}

#[test]
fn multiplelambeu() {
  let mut compiler = Compiler::new(
    "tests/multiplelambeu.robson".to_owned(),
    Box::new(TestInfra::new("".to_owned())),
  )
  .unwrap();
  let compiled = compiler.compile().unwrap();
  let mut infra = TestInfra::new("12\ntesteteste123".to_owned());

  let mut interpreter = Interpreter::<10>::new(&compiled);
  interpreter.run_buffer(&mut infra).unwrap()
}

#[test]
fn convert_robson_macro() {
  let mut hash = HashMap::new();

  hash.insert("1$ROBSON".to_string(), "'comeu 32'".to_string());

  let (a, a_, _) =
    convert_macro_robson("1$ROBSON".to_string(), &hash, 0).unwrap();

  assert_eq!(&a, "'comeu 32'");
  assert_eq!(a_, false);

  let (b, b_, _) =
    convert_macro_robson("i'rcc 1$ROBSON".to_string(), &hash, 0)
      .unwrap();
  assert_eq!(&b, "comeu 99\n");
  assert_eq!(b_, true);

  let (c, c_, _) =
    convert_macro_robson("i'rcc 1$ROBSON".to_string(), &hash, 1)
      .unwrap();

  assert_eq!(&c, "comeu 111\n");
  assert_eq!(c_, true);
}

#[test]
fn thousand() {
  let mut compiler = Compiler::new(
    "tests/1000.robson".to_owned(),
    Box::new(TestInfra::new("".to_owned())),
  )
  .unwrap();
  let compiled = compiler.compile().unwrap();
  let mut infra = TestInfra::new("".to_owned());

  let mut interpreter = Interpreter::<10>::new(&compiled);
  interpreter.run_buffer(&mut infra).unwrap()
}
