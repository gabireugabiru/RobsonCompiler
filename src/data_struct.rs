use std::{
  error::Error,
  fmt::Display,
  num::{ParseFloatError, ParseIntError},
  ops::{Deref, DerefMut},
};

use crate::utils::{
  f32_from_bytes, f32_to_bytes, i32_from_bytes, i32_to_bytes,
  u32_from_bytes, u32_to_bytes,
};

// #[derive(Debug, Clone, Copy)]
// pub struct InstructionParam {
//   pub convert: bool,
//   pub kind: usize,
//   pub byte: TypedByte,
// }

// impl InstructionParam {
//   pub fn fn_is_empty(&self) -> bool {
//     (*self.byte).is_empty() && self.kind == 0 && self.convert
//   }
// }
// impl Display for InstructionParam {
//   fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//     write!(
//       f,
//       "{} {} {}",
//       match self.kind {
//         0 => "comeu",
//         1 => "chupou",
//         2 => "fudeu",
//         3 => "penetrou",
//         _ => "unknwon",
//       },
//       self.byte,
//       if self.convert { "robson" } else { "" }
//     )
//   }
// }
#[derive(Debug, Clone, Copy)]
pub struct Instruction {
  pub opcode: u8,
  pub params: [(TypedByte, usize, bool); 3],
}
impl Display for Instruction {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut params = String::new();

    for (index, i) in self.params.iter().enumerate() {
      params.push_str(&format!(
        "param{} {}\n",
        index + 1,
        match i.1 {
          0 => "comeu",
          1 => "chupou",
          2 => "fudeu",
          3 => "penetrou",
          _ => "unknown",
        }
      ));
    }

    write!(f, "opcode: {}\n{params}", self.opcode)
  }
}
impl Instruction {
  pub const fn new() -> Self {
    Self {
      opcode: 0,
      params: [
        (
          TypedByte {
            r#type: Type::Usigned,
            value: [0; 4],
          },
          0,
          false,
        ),
        (
          TypedByte {
            r#type: Type::Usigned,
            value: [0; 4],
          },
          0,
          false,
        ),
        (
          TypedByte {
            r#type: Type::Usigned,
            value: [0; 4],
          },
          0,
          false,
        ),
      ],
    }
  }
}
#[derive(Debug, Clone)]
pub struct IError {
  pub error: String,
}
impl IError {
  pub fn message<T>(error: T) -> Self
  where
    T: Display,
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
      value: u32_to_bytes(value),
      r#type: Type::Usigned,
    }
  }
}
impl From<i32> for TypedByte {
  fn from(value: i32) -> Self {
    Self {
      value: i32_to_bytes(value),
      r#type: Type::Signed,
    }
  }
}
impl From<f32> for TypedByte {
  fn from(value: f32) -> Self {
    Self {
      value: f32_to_bytes(value),
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
impl From<usize> for TypedByte {
  fn from(value: usize) -> Self {
    Self {
      value: u32_to_bytes(value as u32),
      r#type: Type::Usigned,
    }
  }
}
impl Into<usize> for TypedByte {
  fn into(self) -> usize {
    if self.r#type == Type::Usigned {
      u32_from_bytes(self.value) as usize
    } else {
      0
    }
  }
}
impl From<bool> for TypedByte {
  fn from(value: bool) -> Self {
    TypedByte {
      value: u32_to_bytes(value as u32),
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
        Type::Usigned => format!("{}", u32_from_bytes(self.value)),
        Type::Signed => format!("i{}", i32_from_bytes(self.value)),
        Type::Floating => format!("f{}", f32_from_bytes(self.value)),
      }
    )
  }
}

impl TypedByte {
  pub fn force_u32(&self) -> u32 {
    u32_from_bytes(self.value)
  }
  pub fn convert(&mut self, to_convert: Type) {
    self.r#type = to_convert;

    match to_convert {
      Type::Usigned => match self.r#type {
        Type::Usigned => {}
        Type::Signed => {
          self.value = u32_to_bytes(i32_from_bytes(self.value) as u32)
        }
        Type::Floating => {
          self.value = u32_to_bytes(f32_from_bytes(self.value) as u32)
        }
      },
      Type::Signed => match self.r#type {
        Type::Usigned => {
          self.value = i32_to_bytes(u32_from_bytes(self.value) as i32)
        }
        Type::Signed => {}
        Type::Floating => {
          self.value = i32_to_bytes(f32_from_bytes(self.value) as i32)
        }
      },
      Type::Floating => match self.r#type {
        Type::Usigned => {
          self.value = f32_to_bytes(u32_from_bytes(self.value) as f32)
        }
        Type::Signed => {
          self.value = f32_to_bytes(i32_from_bytes(self.value) as f32)
        }
        Type::Floating => {}
      },
    }
  }
}

#[derive(Debug)]
pub struct Stack<const A: usize> {
  pub vec: [TypedByte; A],
  pub sx: usize,
}
impl<const A: usize> Display for Stack<A> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut string = String::new();
    for i in &self.vec {
      string.push_str(&format!("{i:?}, "));
    }
    write!(f, "[{}]", string)
  }
}
impl<const A: usize> Deref for Stack<A> {
  type Target = [TypedByte; A];
  fn deref(&self) -> &Self::Target {
    &self.vec
  }
}
impl<const A: usize> DerefMut for Stack<A> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.vec
  }
}
impl<const A: usize> Stack<A> {
  pub const fn new() -> Self {
    Self {
      sx: 0,
      vec: [TypedByte {
        value: [0; 4],
        r#type: Type::Usigned,
      }; A],
    }
  }

  pub fn top(&self) -> TypedByte {
    self.vec[self.sx]
  }
  pub fn pop(&mut self) {
    self.sx = self.sx.max(1) - 1;
  }
  pub fn push(&mut self, a: TypedByte) {
    self.sx += 1;
    self.vec[self.sx] = a;
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
