pub fn u32_add(a: [u8; 4], b: [u8; 4]) -> u32 {
  u32::from_be_bytes(a) + u32::from_be_bytes(b)
}

pub fn f32_add(a: [u8; 4], b: [u8; 4]) -> f32 {
  (f32::from_be_bytes(a) + f32::from_be_bytes(b)) as f32
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
