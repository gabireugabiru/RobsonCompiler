use crate::{compiler::Compiler, interpreter::Interpreter, Infra};

pub struct TestInfra {
  stdin: String,
  stdout: String,
}
impl TestInfra {
  fn new(stdin: String) -> Box<Self> {
    Box::new(Self {
      stdin,
      stdout: String::new(),
    })
  }
}
impl Infra for TestInfra {
  fn print(&mut self, to_print: String) {
    self.stdout.push_str(&to_print);
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
    &self,
  ) -> Result<(), crate::data_struct::IError> {
    Ok(())
  }
  fn enable_raw_mode(
    &self,
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
  fn clone_self(&mut self) -> Box<dyn Infra> {
    Box::new(TestInfra {
      stdout: self.stdout.clone(),
      stdin: self.stdout.clone(),
    })
  }
  fn color_print(&mut self, to_print: String, _color: u32) {
    self.print(to_print)
  }
}

#[test]
fn push_and_print() {
  let mut compiler = Compiler::new(
    "tests/push.robson".to_owned(),
    TestInfra::new("".to_owned()),
  )
  .unwrap();
  let compiled = compiler.compile().unwrap();

  let mut interpreter =
    Interpreter::new(&compiled, TestInfra::new(String::new()))
      .unwrap();
  while !interpreter.run_buffer().unwrap() {}
}
#[test]
fn jump() {
  let mut compiler = Compiler::new(
    "tests/jump.robson".to_owned(),
    TestInfra::new("".to_owned()),
  )
  .unwrap();
  let compiled = compiler.compile().unwrap();

  let mut interpreter =
    Interpreter::new(&compiled, TestInfra::new(String::new()))
      .unwrap();
  while !interpreter.run_buffer().unwrap() {}
}

#[test]
fn memory() {
  let mut compiler = Compiler::new(
    "tests/memory.robson".to_owned(),
    TestInfra::new("".to_owned()),
  )
  .unwrap();
  let compiled = compiler.compile().unwrap();
  let mut interpreter =
    Interpreter::new(&compiled, TestInfra::new(String::new()))
      .unwrap();
  while !interpreter.run_buffer().unwrap() {}
}

#[test]
fn if_() {
  let mut compiler = Compiler::new(
    "tests/if.robson".to_owned(),
    TestInfra::new("".to_owned()),
  )
  .unwrap();
  let compiled = compiler.compile().unwrap();

  let mut interpreter =
    Interpreter::new(&compiled, TestInfra::new(String::new()))
      .unwrap();
  while !interpreter.run_buffer().unwrap() {}
}
#[test]
fn input() {
  let mut compiler = Compiler::new(
    "tests/input.robson".to_owned(),
    TestInfra::new("".to_owned()),
  )
  .unwrap();
  let compiled = compiler.compile().unwrap();
  let mut interpreter = Interpreter::new(
    &compiled,
    TestInfra::new("12\ntesteteste123".to_owned()),
  )
  .unwrap();
  while !interpreter.run_buffer().unwrap() {}
}
#[test]
fn operations() {
  let mut compiler = Compiler::new(
    "tests/operations.robson".to_owned(),
    TestInfra::new("".to_owned()),
  )
  .unwrap();
  let compiled = compiler.compile().unwrap();

  let mut interpreter =
    Interpreter::new(&compiled, TestInfra::new(String::new()))
      .unwrap();
  while !interpreter.run_buffer().unwrap() {}
}

#[test]
fn types() {
  let mut compiler = Compiler::new(
    "tests/types.robson".to_owned(),
    TestInfra::new("".to_owned()),
  )
  .unwrap();
  let compiled = compiler.compile().unwrap();
  let mut interpreter =
    Interpreter::new(&compiled, TestInfra::new(String::new()))
      .unwrap();
  while !interpreter.run_buffer().unwrap() {}
}

#[test]
fn include() {
  let mut compiler = Compiler::new(
    "tests/include.robson".to_owned(),
    TestInfra::new("".to_owned()),
  )
  .unwrap();
  let compiled = compiler.compile().unwrap();
  let mut interpreter =
    Interpreter::new(&compiled, TestInfra::new(String::new()))
      .unwrap();
  while !interpreter.run_buffer().unwrap() {}
}
