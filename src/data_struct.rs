use std::{
  error::Error,
  fmt::Display,
  num::{ParseFloatError, ParseIntError},
  ops::{Deref, DerefMut},
};

#[derive(Debug)]
pub struct IError {
  error: String,
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
      error: format!("{err}"),
    }
  }
}
impl From<ParseFloatError> for IError {
  fn from(err: ParseFloatError) -> Self {
    Self {
      error: format!("{err}"),
    }
  }
}
impl From<std::io::Error> for IError {
  fn from(err: std::io::Error) -> Self {
    Self {
      error: format!("{err}"),
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

impl Into<u32> for TypedByte {
  fn into(self) -> u32 {
    match self.r#type {
      Type::Usigned => u32::from_be_bytes(self.value),
      Type::Signed => i32::from_be_bytes(self.value) as u32,
      Type::Floating => f32::from_be_bytes(self.value) as u32,
    }
  }
}
impl Into<i32> for TypedByte {
  fn into(self) -> i32 {
    match self.r#type {
      Type::Usigned => u32::from_be_bytes(self.value) as i32,
      Type::Signed => i32::from_be_bytes(self.value),
      Type::Floating => f32::from_be_bytes(self.value) as i32,
    }
  }
}
impl Into<f32> for TypedByte {
  fn into(self) -> f32 {
    match self.r#type {
      Type::Usigned => u32::from_be_bytes(self.value) as f32,
      Type::Signed => i32::from_be_bytes(self.value) as f32,
      Type::Floating => f32::from_be_bytes(self.value),
    }
  }
}
impl Into<bool> for TypedByte {
  fn into(self) -> bool {
    match self.r#type {
      Type::Usigned => u32::from_be_bytes(self.value) != 0,
      Type::Signed => i32::from_be_bytes(self.value) != 0,
      Type::Floating => f32::from_be_bytes(self.value) != 0.0,
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
  pub fn force_u32(
    &self,
    current_command: usize,
  ) -> Result<u32, IError> {
    if self.r#type != Type::Usigned {
      return Err(IError::message(format!(
        "Invalid number type at the command {current_command}"
      )));
    }
    Ok(u32::from_be_bytes(self.value))
  }
  pub fn convert(self, to_convert: Type) -> TypedByte {
    match to_convert {
      Type::Usigned => match self.r#type {
        Type::Floating => {
          (f32::from_be_bytes(self.value) as u32).into()
        }
        Type::Signed => {
          (i32::from_be_bytes(self.value) as u32).into()
        }
        Type::Usigned => self,
      },
      Type::Signed => match self.r#type {
        Type::Floating => {
          (f32::from_be_bytes(self.value) as i32).into()
        }
        Type::Signed => self,
        Type::Usigned => {
          (u32::from_be_bytes(self.value) as i32).into()
        }
      },
      Type::Floating => match self.r#type {
        Type::Floating => self,
        Type::Signed => {
          (i32::from_be_bytes(self.value) as f32).into()
        }
        Type::Usigned => {
          (u32::from_be_bytes(self.value) as f32).into()
        }
      },
    }
  }
}

#[derive(Default, Debug)]
pub struct Stack {
  pub vec: Vec<TypedByte>,
}
impl Display for Stack {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut string = String::new();
    for i in &self.vec {
      string.push_str(&format!("{i}, "));
    }
    write!(f, "[{}]", string)
  }
}
impl Deref for Stack {
  type Target = Vec<TypedByte>;
  fn deref(&self) -> &Self::Target {
    &self.vec
  }
}
impl DerefMut for Stack {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.vec
  }
}
impl Stack {
  pub fn top(&self) -> Result<TypedByte, IError> {
    if !self.vec.is_empty() {
      return Ok(self.vec[self.len() - 1]);
    }
    Err(IError::message(
      "Trying to access the stack while it is empty",
    ))
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
