use std::{
  error::Error,
  fmt::Display,
  num::{ParseFloatError, ParseIntError},
  ops::{Deref, DerefMut},
};

#[derive(Debug)]
pub struct InstructionParam {
  pub convert: bool,
  pub kind: usize,
  pub byte: TypedByte,
}

impl InstructionParam {
  pub fn fn_is_empty(&self) -> bool {
    (*self.byte).is_empty() && self.kind == 0 && self.convert
  }
}
impl Display for InstructionParam {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{} {} {}",
      match self.kind {
        0 => "comeu",
        1 => "chupou",
        2 => "fudeu",
        3 => "penetrou",
        _ => "unknwon",
      },
      self.byte,
      if self.convert { "robson" } else { "" }
    )
  }
}
#[derive(Debug)]
pub struct Instruction {
  pub opcode: u8,
  pub params: [InstructionParam; 3],
}
impl Display for Instruction {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut params = String::new();

    for (index, i) in self.params.iter().enumerate() {
      if !i.fn_is_empty() {
        params.push_str(&format!("param{} {}\n", index + 1, i));
      }
    }

    write!(f, "opcode: {}\n{params}", self.opcode)
  }
}
#[derive(Debug)]
pub struct IError {
  pub error: String,
}
impl IError {
  pub fn message<T>(error: T) -> Self
  where
    T: ToString,
  {
    Self {
      error: error.to_string(),
    }
  }
}
impl From<ParseIntError> for IError {
  fn from(err: ParseIntError) -> Self {
    Self {
      error: err.to_string(),
    }
  }
}
impl From<ParseFloatError> for IError {
  fn from(err: ParseFloatError) -> Self {
    Self {
      error: err.to_string(),
    }
  }
}
impl From<std::io::Error> for IError {
  fn from(err: std::io::Error) -> Self {
    Self {
      error: err.to_string(),
    }
  }
}
impl Display for IError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.error)
  }
}
impl Error for IError {}
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct TypedByte {
  pub value: [u8; 4],
  pub r#type: Type,
}
impl From<u32> for TypedByte {
  fn from(value: u32) -> Self {
    Self {
      value: value.to_be_bytes(),
      r#type: Type::Usigned,
    }
  }
}
impl From<i32> for TypedByte {
  fn from(value: i32) -> Self {
    Self {
      value: value.to_be_bytes(),
      r#type: Type::Signed,
    }
  }
}
impl From<f32> for TypedByte {
  fn from(value: f32) -> Self {
    Self {
      value: value.to_be_bytes(),
      r#type: Type::Floating,
    }
  }
}

impl From<[u8; 4]> for TypedByte {
  fn from(value: [u8; 4]) -> Self {
    Self {
      value,
      r#type: Type::Usigned,
    }
  }
}

impl From<bool> for TypedByte {
  fn from(value: bool) -> Self {
    TypedByte {
      value: (value as u32).to_be_bytes(),
      r#type: Type::Usigned,
    }
  }
}

impl Deref for TypedByte {
  type Target = [u8; 4];
  fn deref(&self) -> &Self::Target {
    &self.value
  }
}
impl Display for TypedByte {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}",
      match self.r#type {
        Type::Usigned =>
          format!("{}", u32::from_be_bytes(self.value)),
        Type::Signed =>
          format!("i{}", i32::from_be_bytes(self.value)),
        Type::Floating =>
          format!("f{}", f32::from_be_bytes(self.value)),
      }
    )
  }
}

impl TypedByte {
  pub fn force_u32(&self) -> Option<u32> {
    if self.r#type != Type::Usigned {
      None
    } else {
      Some(u32::from_be_bytes(self.value))
    }
  }
  pub fn convert(&mut self, to_convert: Type) {
    self.r#type = to_convert;

    match to_convert {
      Type::Usigned => match self.r#type {
        Type::Usigned => {}
        Type::Signed => {
          self.value =
            (i32::from_be_bytes(self.value) as u32).to_be_bytes()
        }
        Type::Floating => {
          self.value =
            (f32::from_be_bytes(self.value) as u32).to_be_bytes()
        }
      },
      Type::Signed => match self.r#type {
        Type::Usigned => {
          self.value =
            (u32::from_be_bytes(self.value) as i32).to_be_bytes()
        }
        Type::Signed => {}
        Type::Floating => {
          self.value =
            (f32::from_be_bytes(self.value) as i32).to_be_bytes()
        }
      },
      Type::Floating => match self.r#type {
        Type::Usigned => {
          self.value =
            (u32::from_be_bytes(self.value) as f32).to_be_bytes()
        }
        Type::Signed => {
          self.value =
            (i32::from_be_bytes(self.value) as f32).to_be_bytes()
        }
        Type::Floating => {}
      },
    }
  }
}

#[derive(Default, Debug)]
pub struct Stack<T> {
  pub vec: Vec<T>,
}
impl<T: std::fmt::Debug> Display for Stack<T> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut string = String::new();
    for i in &self.vec {
      string.push_str(&format!("{i:?}, "));
    }
    write!(f, "[{}]", string)
  }
}
impl<T> Deref for Stack<T> {
  type Target = Vec<T>;
  fn deref(&self) -> &Self::Target {
    &self.vec
  }
}
impl<T> DerefMut for Stack<T> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.vec
  }
}
impl<T: Copy> Stack<T> {
  pub fn top(&self) -> Option<T> {
    if !self.vec.is_empty() {
      return Some(self.vec[self.len() - 1]);
    }
    None
  }
}

const TYPES: [Type; 3] =
  [Type::Usigned, Type::Signed, Type::Floating];

#[derive(Clone, Copy, PartialEq, Debug, Eq, PartialOrd)]
pub enum Type {
  Usigned = 0,
  Signed = 1,
  Floating = 2,
}

impl From<usize> for Type {
  fn from(value: usize) -> Self {
    TYPES[value]
  }
}
impl Default for Type {
  fn default() -> Self {
    Self::Usigned
  }
}
