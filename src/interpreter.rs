use std::{
  io::Write,
  time::{Duration, Instant},
};

use rand::Rng;

use crate::{
  data_struct::{
    IError, Instruction, InstructionParam, Stack, Type, TypedByte,
  },
  macros::{convert, force_u32, ierror, top},
  utils::{
    convert_kind_byte, convert_two_bits, f32_mod, i32_mod, u32_mod,
  },
  Infra,
};

use super::utils::{
  approx_equal, f32_add, f32_div, f32_mul, f32_sub, i32_add, i32_div,
  i32_mul, i32_sub, u32_add, u32_div, u32_mul, u32_sub,
};

pub const OPERATIONS: [[fn(&mut Interpreter, (TypedByte, TypedByte));
  3]; 5] = [
  [
    |int, (v1, v2)| int.stack.push(u32_add(*v1, *v2).into()),
    |int, (v1, v2)| int.stack.push(i32_add(*v1, *v2).into()),
    |int, (v1, v2)| int.stack.push(f32_add(*v1, *v2).into()),
  ],
  [
    |int, (v1, v2)| {
      int.stack.push({
        if *v1 >= *v2 {
          u32_sub(*v1, *v2).into()
        } else {
          0u32.into()
        }
      })
    },
    |int, (v1, v2)| int.stack.push(i32_sub(*v1, *v2).into()),
    |int, (v1, v2)| int.stack.push(f32_sub(*v1, *v2).into()),
  ],
  [
    |int, (v1, v2)| int.stack.push(u32_mul(*v1, *v2).into()),
    |int, (v1, v2)| int.stack.push(i32_mul(*v1, *v2).into()),
    |int, (v1, v2)| int.stack.push(f32_mul(*v1, *v2).into()),
  ],
  [
    |int, (v1, v2)| int.stack.push(u32_div(*v1, *v2).into()),
    |int, (v1, v2)| int.stack.push(i32_div(*v1, *v2).into()),
    |int, (v1, v2)| int.stack.push(f32_div(*v1, *v2).into()),
  ],
  [
    |int, (v1, v2)| int.stack.push(u32_mod(*v1, *v2).into()),
    |int, (v1, v2)| int.stack.push(i32_mod(*v1, *v2).into()),
    |int, (v1, v2)| int.stack.push(f32_mod(*v1, *v2).into()),
  ],
];

pub struct Interpreter<'a> {
  pub memory: Vec<TypedByte>,
  pub debug: bool,
  pub stack: Stack<TypedByte>,
  time: Option<Instant>,
  duration: Option<Duration>,
  infra: &'a mut dyn Infra,
  index: usize,
  current_command: usize,
  buffer: &'a [u8],
  opcode_functions: [fn(
    &mut Interpreter,
    params: [(TypedByte, usize, bool); 3],
  ) -> Result<(), IError>; 17],
  convertion_array:
    [fn(TypedByte, &mut Interpreter) -> Option<TypedByte>; 4],
}
#[inline(always)]
fn not_convert(
  byte: TypedByte,
  _interpreter: &mut Interpreter,
) -> Option<TypedByte> {
  Some(byte)
}

#[inline(always)]
fn convert_chupou(
  _: TypedByte,
  interpreter: &mut Interpreter,
) -> Option<TypedByte> {
  let top = interpreter.stack.top()?;
  interpreter.stack.pop();
  Some(top)
}

#[inline(always)]
fn conver_fudeu(
  byte: TypedByte,
  interpreter: &mut Interpreter,
) -> Option<TypedByte> {
  let address = byte.force_u32()? as usize;
  interpreter.validate_until(address);

  Some(interpreter.memory[address])
}

#[inline(always)]
fn convert_penetrou(
  _: TypedByte,
  interpreter: &mut Interpreter,
) -> Option<TypedByte> {
  let address = interpreter.stack.top()?.force_u32()? as usize;
  interpreter.stack.pop();

  interpreter.validate_until(address);

  Some(interpreter.memory[address])
}

fn do_no_shit(
  _: &mut Interpreter,
  _: [(TypedByte, usize, bool); 3],
) -> Result<(), IError> {
  ierror!("Operation is not operating, please suck my dick")
}
// OPCODE 1
#[inline(always)]
fn operations(
  interpreter: &mut Interpreter,
  [param1, param2, param3]: [(TypedByte, usize, bool); 3],
) -> Result<(), IError> {
  let kind = force_u32!(interpreter, convert!(interpreter, param1)?)?;

  let mut value = convert!(interpreter, param2)?;
  let mut value2 = convert!(interpreter, param3)?;

  if param2.2 {
    value.convert(value2.r#type);
  }

  if param3.2 {
    value2.convert(value.r#type);
  }

  if value.r#type != value2.r#type {
    return ierror!(
      "Adding with incompatible types at command {}",
      interpreter.current_command
    );
  }
  OPERATIONS[kind as usize][value.r#type as usize](
    interpreter,
    (value, value2),
  );
  Ok(())
}

//OPCODE 2
#[inline(always)]
fn if_lower(
  interpreter: &mut Interpreter,
  [param1, param2, param3]: [(TypedByte, usize, bool); 3],
) -> Result<(), IError> {
  let value = convert!(interpreter, param1)?;
  let value2 = convert!(interpreter, param2)?;

  let pos = force_u32!(interpreter, convert!(interpreter, param3)?)?;
  if value.r#type != value2.r#type {
    return Err(IError::message(format!(
      "Comparing with incompatible types {}",
      interpreter.current_command
    )));
  }
  if *value < *value2 {
    interpreter.index = (pos * 15) as usize;
  }
  Ok(())
}

//OPCODE 3
fn push_stack(
  interpreter: &mut Interpreter,
  [param1, ..]: [(TypedByte, usize, bool); 3],
) -> Result<(), IError> {
  let value = convert!(interpreter, param1)?;
  interpreter.stack.push(value);
  Ok(())
}

//OPCODE 4
fn if_true_jump(
  interpreter: &mut Interpreter,
  [param1, param2, param3]: [(TypedByte, usize, bool); 3],
) -> Result<(), IError> {
  let value = convert!(interpreter, param1)?;
  let value2 = convert!(interpreter, param2)?;
  let pos = force_u32!(interpreter, convert!(interpreter, param3)?)?;

  if value.r#type != value2.r#type {
    return ierror!("Comparing incompatible types");
  }

  if value.r#type == Type::Floating {
    let value = f32::from_be_bytes(*value);
    let value2 = f32::from_be_bytes(*value2);
    if approx_equal(value, value2, 4) {
      interpreter.index = (pos * 15) as usize;
    }
  } else if *value == *value2 {
    interpreter.index = (pos * 15) as usize;
  };
  Ok(())
}

//OPCODE 5
#[inline(always)]
fn vstack_jump(
  interpreter: &mut Interpreter,
  [param1, ..]: [(TypedByte, usize, bool); 3],
) -> Result<(), IError> {
  let value =
    force_u32!(interpreter, convert!(interpreter, param1)?)?;
  if interpreter.stack.is_empty() {
    interpreter.index = (value * 15) as usize;
  }
  Ok(())
}

//OPCODE 6
#[inline(always)]
fn input(
  interpreter: &mut Interpreter,
  [param1, param2, param3]: [(TypedByte, usize, bool); 3],
) -> Result<(), IError> {
  let mut value =
    force_u32!(interpreter, convert!(interpreter, param1)?)? as usize;
  let kind = force_u32!(interpreter, convert!(interpreter, param2)?)?;
  let limit =
    force_u32!(interpreter, convert!(interpreter, param3)?)? as usize;

  std::io::stdout().flush()?;
  let buff = interpreter.infra.read_line()?;

  match kind {
    1 => {
      interpreter.validate_until(value);
      match buff.trim().parse::<u32>() {
        Ok(x) => interpreter.memory[value] = x.into(),
        Err(_) => interpreter.stack.push(1u32.into()),
      };
    }
    2 => {
      interpreter.validate_until(value);
      match buff.trim().parse::<i32>() {
        Ok(x) => interpreter.memory[value] = x.into(),
        Err(_) => interpreter.stack.push(1u32.into()),
      }
      // interpreter.memory[value] = buff.trim().parse::<i32>()?.into()
    }
    3 => {
      interpreter.validate_until(value);
      match buff.trim().parse::<f32>() {
        Ok(x) => interpreter.memory[value] = x.into(),
        Err(_) => interpreter.stack.push(1u32.into()),
      }
      // interpreter.memory[value] = buff.trim().parse::<f32>()?.into()
    }
    _ => {
      let address_to = value + limit + 2;
      interpreter.validate_until(address_to);
      for (i, char) in buff.chars().enumerate() {
        if i < limit {
          let char = if char == '\n' || char == '\0' {
            interpreter.memory[value] = [0; 4].into();
            continue;
          } else {
            char
          };
          let mut bytes: [u8; 4] = [0, 0, 0, 0];

          char.encode_utf8(&mut bytes);
          let mut zeroes = 0;
          for a in bytes {
            if a == 0 {
              zeroes += 1;
            }
          }
          interpreter.memory[value] =
            (u32::from_be_bytes(bytes) >> (8 * zeroes)).into();
          value += 1;
        } else {
          break;
        }
      }
      interpreter.memory[value] = 0u32.into();
    }
  };

  Ok(())
}

//OPCODE 7
#[inline(always)]
fn print(
  interpreter: &mut Interpreter,
  [..]: [(TypedByte, usize, bool); 3],
) -> Result<(), IError> {
  if interpreter.stack.is_empty() {
    return Err(IError::message(format!(
      "Trying to use the stack while empty in command {}",
      interpreter.current_command
    )));
  }
  let stack_byte = top!(interpreter.stack)?;
  if stack_byte.r#type != Type::Usigned {
    return Err(IError::message("Invalid number type for ASCII"));
  }
  interpreter.infra.print(&*stack_byte);

  interpreter.stack.pop();
  Ok(())
}

//OPCODE 8
#[inline(always)]
fn printnumber(
  interpreter: &mut Interpreter,
  [..]: [(TypedByte, usize, bool); 3],
) -> Result<(), IError> {
  let TypedByte { value, r#type } = top!(interpreter.stack)?;

  match r#type {
    Type::Floating => interpreter
      .infra
      .print(f32::from_be_bytes(value).to_string().as_bytes()),
    Type::Signed => interpreter
      .infra
      .print(i32::from_be_bytes(value).to_string().as_bytes()),
    Type::Usigned => interpreter
      .infra
      .print(u32::from_be_bytes(value).to_string().as_bytes()),
  }

  interpreter.stack.pop();
  Ok(())
}

//OPCODE 9
#[inline(always)]
fn jump(
  interpreter: &mut Interpreter,
  [param1, ..]: [(TypedByte, usize, bool); 3],
) -> Result<(), IError> {
  let value =
    force_u32!(interpreter, convert!(interpreter, param1)?)?;
  interpreter.index = (value * 15) as usize;
  Ok(())
}

//OPCODE 10
#[inline(always)]
fn set(
  interpreter: &mut Interpreter,
  [param1, ..]: [(TypedByte, usize, bool); 3],
) -> Result<(), IError> {
  let address =
    force_u32!(interpreter, convert!(interpreter, param1)?)? as usize;
  let typed_byte = top!(interpreter.stack)?;

  interpreter.stack.pop();
  interpreter.validate_until(address);
  interpreter.memory[address] = typed_byte;
  Ok(())
}

//OPCODE 11
#[inline(always)]
fn pop_stack(
  interpreter: &mut Interpreter,
  [..]: [(TypedByte, usize, bool); 3],
) -> Result<(), IError> {
  if !interpreter.stack.is_empty() {
    interpreter.stack.pop();
  }
  Ok(())
}

//OPCODE 12
#[inline(always)]
fn load_string(
  interpreter: &mut Interpreter,
  [param1, ..]: [(TypedByte, usize, bool); 3],
) -> Result<(), IError> {
  let mut value =
    force_u32!(interpreter, convert!(interpreter, param1)?)? as usize;
  let mut buffer: Vec<u32> = Vec::new();
  interpreter.validate_until(value + 5);
  loop {
    if value == interpreter.memory.len() {
      break;
    }
    let temp = u32::from_be_bytes(*interpreter.memory[value]);
    if temp != 0 {
      buffer.push(temp);
      value += 1;
    } else {
      break;
    }
  }
  buffer.reverse();
  for i in buffer {
    interpreter.stack.push(TypedByte {
      value: i.to_be_bytes(),
      r#type: Type::Usigned,
    });
  }
  Ok(())
}

//OPCODE 13
#[inline(always)]
fn time_operations(
  interpreter: &mut Interpreter,
  [param1, ..]: [(TypedByte, usize, bool); 3],
) -> Result<(), IError> {
  match force_u32!(interpreter, convert!(interpreter, param1)?)? {
    // SET ax
    0 => {
      interpreter.time = Some(Instant::now());
    }
    //SET bx
    1 => {
      let a = top!(interpreter.stack)?.value;
      interpreter.stack.pop();
      let b = top!(interpreter.stack)?.value;
      let result = [a[0], a[1], a[2], a[3], b[0], b[1], b[2], b[3]];

      interpreter.duration =
        Some(Duration::from_millis(u64::from_be_bytes(result)))
    }
    // CMP ax elapsed to bx
    2 => {
      if let Some(a) = interpreter.time {
        if let Some(b) = interpreter.duration {
          let elapsed = a.elapsed();
          match elapsed.cmp(&b) {
            std::cmp::Ordering::Less => {
              interpreter.stack.push(0u32.into())
            }
            std::cmp::Ordering::Equal => {
              interpreter.stack.push(1u32.into())
            }
            std::cmp::Ordering::Greater => {
              interpreter.stack.push(2u32.into())
            }
          }
        }
      }
    }
    _ => {}
  }
  Ok(())
}
//OPCODE 14
#[inline(always)]
fn flush(
  interpreter: &mut Interpreter,
  _: [(TypedByte, usize, bool); 3],
) -> Result<(), IError> {
  interpreter.infra.flush();
  Ok(())
}
//OPCODE 15
#[inline(always)]
fn terminal_commands(
  interpreter: &mut Interpreter,
  [param1, ..]: [(TypedByte, usize, bool); 3],
) -> Result<(), IError> {
  match force_u32!(interpreter, convert!(interpreter, param1)?)? {
    // RAW MODE
    0 => {
      let on_off = force_u32!(interpreter, top!(interpreter.stack)?)?;
      interpreter.stack.pop();
      if on_off == 0 {
        interpreter.infra.disable_raw_mode()?;
      } else {
        interpreter.infra.enable_raw_mode()?;
      }
    }
    // CLEAR
    1 => {
      let r#type = force_u32!(interpreter, top!(interpreter.stack)?)?;
      interpreter.stack.pop();
      if r#type == 0 {
        interpreter.infra.clear_purge()?;
      } else {
        interpreter.infra.clear_all()?;
      }
    }
    // POLL KEYBOARD
    2 => {
      let a = *top!(interpreter.stack)?;
      interpreter.stack.pop();
      let b = *top!(interpreter.stack)?;
      interpreter.stack.pop();
      let result = [a[0], a[1], a[2], a[3], b[0], b[1], b[2], b[3]];
      let value =
        interpreter.infra.poll(u64::from_be_bytes(result))?;

      interpreter.stack.push(value.into());
    }
    // SHOW/HIDE CURSOR
    3 => {
      let on_off = force_u32!(interpreter, top!(interpreter.stack)?)?;
      interpreter.stack.pop();
      if on_off == 0 {
        interpreter.infra.hide_cursor()?;
      } else {
        interpreter.infra.show_cursor()?;
      }
    }
    // MOVE CURSOR
    4 => {
      let x = force_u32!(interpreter, top!(interpreter.stack)?)?;
      interpreter.stack.pop();
      let y = force_u32!(interpreter, top!(interpreter.stack)?)?;
      interpreter.stack.pop();
      interpreter.infra.move_cursor(x, y)?;
    }
    // FONT COLOR
    5 => {
      let color = force_u32!(interpreter, top!(interpreter.stack)?)?;
      interpreter.stack.pop();
      interpreter.infra.use_color(color)?;
    }
    //BACKGROUND
    6 => {
      let color = force_u32!(interpreter, top!(interpreter.stack)?)?;
      interpreter.stack.pop();
      interpreter.infra.use_background(color)?;
    }
    _ => return Err(IError::message("Invalid terminal command")),
  }
  Ok(())
}

// OPCODE 16
fn random(
  interpreter: &mut Interpreter,
  _: [(TypedByte, usize, bool); 3],
) -> Result<(), IError> {
  let mut rng = rand::thread_rng();
  interpreter.stack.push(rng.gen::<f32>().into());
  Ok(())
}

impl<'a> Interpreter<'a> {
  pub fn new(
    buffer: &'a [u8],
    infra: &'a mut dyn Infra,
  ) -> Result<Self, IError> {
    Ok(Self {
      memory: Vec::new(),
      stack: Stack::default(),
      debug: false,
      time: None,
      duration: None,
      infra,
      index: 0,
      buffer,
      current_command: 0,
      opcode_functions: [
        do_no_shit,
        operations,
        if_lower,
        push_stack,
        if_true_jump,
        vstack_jump,
        input,
        print,
        printnumber,
        jump,
        set,
        pop_stack,
        load_string,
        time_operations,
        flush,
        terminal_commands,
        random,
      ],
      convertion_array: [
        not_convert,
        convert_chupou,
        conver_fudeu,
        convert_penetrou,
      ],
    })
  }

  pub fn get_bytes_data(buffer: &[u8]) -> Instruction {
    let opcode = buffer[0];
    let kind_byte = buffer[1];
    let param1_byte = [buffer[2], buffer[3], buffer[4], buffer[5]];
    let param2_byte = [buffer[6], buffer[7], buffer[8], buffer[9]];
    let param3_byte =
      [buffer[10], buffer[11], buffer[12], buffer[13]];
    let types = buffer[14];
    let converted_types = convert_kind_byte(types);
    let converted_kind = convert_kind_byte(kind_byte);

    let param1 = TypedByte {
      value: param1_byte,
      r#type: Type::from(converted_types[0]),
    };
    let param2 = TypedByte {
      value: param2_byte,
      r#type: Type::from(converted_types[1]),
    };
    let param3 = TypedByte {
      value: param3_byte,
      r#type: Type::from(converted_types[2]),
    };

    let [param1_convert, param2_convert] =
      convert_two_bits(converted_kind[3] as u8);
    let [param3_convert, _] =
      convert_two_bits(converted_types[3] as u8);

    Instruction {
      opcode,
      params: [
        InstructionParam {
          byte: param1,
          convert: param1_convert,
          kind: converted_kind[0],
        },
        InstructionParam {
          byte: param2,
          convert: param2_convert,
          kind: converted_kind[1],
        },
        InstructionParam {
          byte: param3,
          convert: param3_convert,
          kind: converted_kind[2],
        },
      ],
    }
  }
  pub fn debug(&mut self, new: bool) {
    self.debug = new;
  }

  pub fn run_buffer(&mut self) -> Result<(), IError> {
    if self.buffer.is_empty() {
      return ierror!("Empty buffer");
    }
    loop {
      if self.index + 15 > self.buffer.len() {
        break;
      }
      let instruction = Self::get_bytes_data(
        &self.buffer[self.index..self.index + 15],
      );
      if self.debug {
        self.infra.println(format!("{}\n", instruction));
      }
      let param1 = &instruction.params[0];
      let param2 = &instruction.params[1];
      let param3 = &instruction.params[2];

      self.index += 15;

      if let Some(err) = self.execute_command(
        instruction.opcode,
        (param1.byte, param1.kind, param1.convert),
        (param2.byte, param2.kind, param2.convert),
        (param3.byte, param3.kind, param3.convert),
      ) {
        return Err(err);
      }

      self.current_command = self.index / 15;
    }
    Ok(())
  }
  pub fn validate_until(&mut self, address: usize) {
    if self.memory.len() <= address {
      self.memory.resize(address + 1, 0u32.into());
    }
  }
  pub fn convert(
    &mut self,
    byte: TypedByte,
    r#type: usize,
  ) -> Option<TypedByte> {
    self.convertion_array[r#type](byte, self)
  }

  #[inline(always)]
  pub fn execute_command(
    &mut self,
    opcode: u8,
    param1: (TypedByte, usize, bool),
    param2: (TypedByte, usize, bool),
    param3: (TypedByte, usize, bool),
  ) -> Option<IError> {
    if let Err(err) = self.opcode_functions[opcode as usize](
      self,
      [param1, param2, param3],
    ) {
      Some(err)
    } else {
      None
    }
  }
}
