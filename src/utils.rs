use std::collections::HashMap;

use crate::{data_struct::IError, macros::ierror};

pub fn u32_add(a: [u8; 4], b: [u8; 4]) -> u32 {
  u32::from_be_bytes(a) + u32::from_be_bytes(b)
}

pub fn f32_add(a: [u8; 4], b: [u8; 4]) -> f32 {
  f32::from_be_bytes(a) + f32::from_be_bytes(b)
}

pub fn i32_add(a: [u8; 4], b: [u8; 4]) -> i32 {
  i32::from_be_bytes(a) + i32::from_be_bytes(b)
}

pub fn u32_sub(a: [u8; 4], b: [u8; 4]) -> u32 {
  u32::from_be_bytes(a) - u32::from_be_bytes(b)
}

pub fn f32_sub(a: [u8; 4], b: [u8; 4]) -> f32 {
  f32::from_be_bytes(a) - f32::from_be_bytes(b)
}

pub fn i32_sub(a: [u8; 4], b: [u8; 4]) -> i32 {
  i32::from_be_bytes(a) - i32::from_be_bytes(b)
}
pub fn u32_mul(a: [u8; 4], b: [u8; 4]) -> u32 {
  u32::from_be_bytes(a) * u32::from_be_bytes(b)
}

pub fn f32_mul(a: [u8; 4], b: [u8; 4]) -> f32 {
  f32::from_be_bytes(a) * f32::from_be_bytes(b)
}

pub fn i32_mul(a: [u8; 4], b: [u8; 4]) -> i32 {
  i32::from_be_bytes(a) * i32::from_be_bytes(b)
}

pub fn u32_div(a: [u8; 4], b: [u8; 4]) -> u32 {
  u32::from_be_bytes(a) / u32::from_be_bytes(b)
}

pub fn f32_div(a: [u8; 4], b: [u8; 4]) -> f32 {
  f32::from_be_bytes(a) / f32::from_be_bytes(b)
}

pub fn i32_div(a: [u8; 4], b: [u8; 4]) -> i32 {
  i32::from_be_bytes(a) / i32::from_be_bytes(b)
}

pub fn u32_mod(a: [u8; 4], b: [u8; 4]) -> u32 {
  u32::from_be_bytes(a) % u32::from_be_bytes(b)
}

pub fn f32_mod(a: [u8; 4], b: [u8; 4]) -> f32 {
  f32::from_be_bytes(a) % f32::from_be_bytes(b)
}

pub fn i32_mod(a: [u8; 4], b: [u8; 4]) -> i32 {
  i32::from_be_bytes(a) % i32::from_be_bytes(b)
}

pub fn approx_equal(a: f32, b: f32, decimal_places: u8) -> bool {
  let factor = 10.0f32.powi(decimal_places as i32);
  let a = (a * factor).trunc();
  let b = (b * factor).trunc();
  a == b
}
pub fn create_kind_byte(
  type1: u8,
  type2: u8,
  type3: u8,
  type4: u8,
) -> u8 {
  let mut kind_byte: u8 = 0;
  kind_byte += type1 * 64;
  kind_byte += type2 * 16;
  kind_byte += type3 * 4;
  kind_byte += type4;
  kind_byte
}
pub fn convert_kind_byte(a: u8) -> [usize; 4] {
  [
    (a >> 6) as usize,
    (((a >> 4) << 6) >> 6) as usize,
    (((a >> 2) << 6) >> 6) as usize,
    ((a << 6) >> 6) as usize,
  ]
}

pub fn create_two_bits(bits: [bool; 2]) -> u8 {
  let mut byte = 0;
  byte += bits[0] as u8;
  byte += bits[1] as u8 * 2;
  byte
}

pub fn convert_two_bits(byte: u8) -> [bool; 2] {
  [((byte << 7) >> 7) != 0, (byte >> 1) != 0]
}

pub fn convert_macro_robson(
  expr: String,
  values: &HashMap<String, String>,
  current: usize,
) -> Result<(String, bool, bool), IError> {
  let splited = expr.split(' ').collect::<Vec<&str>>();

  if splited.len() == 1 {
    return Ok((
      values
        .get(&expr)
        .ok_or(IError::message(format!("Can't find {}", expr)))?
        .to_string(),
      false,
      false,
    ));
  }
  let is_expr = true;
  if splited.len() != 2 {
    return ierror!("Failed to parse '{}'", expr);
  }
  if splited[1].contains("?ROBSON") {
    return ierror!("Cant use an expression with x?ROBSONs");
  }

  let mut value = values
    .get(splited[1])
    .ok_or(IError::message("Malformated macro param"))?
    .to_owned();
  let mut last: Option<char> = None;
  let mut has_next = false;
  let chs = splited[0].chars().collect::<Vec<char>>();

  for (i, char) in chs.iter().enumerate() {
    if let Some(l) = last {
      match l {
        's' => {
          let s = value.split(*char).collect::<Vec<&str>>();
          value = s[current].to_string();
        }
        'i' => {
          let s = value.split(*char).collect::<Vec<&str>>();
          if s.len() != 3 {
            return ierror!(
              "Invalid value inside expression '{}'",
              value
            );
          }

          value = s[1].to_string();
        }
        'c' => {
          let chars = value.chars().collect::<Vec<char>>();
          value = String::new();
          for i in chars {
            let mut bytes: [u8; 4] = [0, 0, 0, 0];

            i.encode_utf8(&mut bytes);
            let mut zeroes = 0;
            for a in bytes {
              if a == 0 {
                zeroes += 1;
              }
            }
            let prefix = match char {
              'c' => "comeu",
              'f' => "fudeu",
              _ => {
                return ierror!(
                  "Invalid complement for 'c' macro expression "
                );
              }
            };
            let number = u32::from_be_bytes(bytes) >> (8 * zeroes);
            value.push_str(&format!("{prefix} {number}\n"));
          }
        }
        _ => return ierror!("Invalid macro expression"),
      }
      last = None;
    } else {
      match char {
        'r' => {
          let chars = value.chars().collect::<Vec<char>>();
          if current >= chars.len() {
            return ierror!("Out of bounds macro expression");
          }
          if current + 1 < chars.len() {
            has_next = true;
          }
          value = chars[current].to_string();
        }
        _ => {
          if (i + 1) >= chs.len() {
            return ierror!("Incomplete '{}' macro argument", char);
          }
          last = Some(*char);
        }
      }
    }
  }

  Ok((value, has_next, is_expr))
}
