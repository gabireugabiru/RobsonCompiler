use std::{
  io::Write,
  time::{Duration, Instant},
};

use rand::Rng;

use crate::{
  data_struct::{IError, Instruction, Stack, Type, TypedByte},
  macros::{convert, force_u32, someierror, top, try_err},
  utils::{
    convert_kind_byte, convert_two_bits, f32_from_bytes, f32_mod,
    f32_to_bytes, i32_from_bytes, i32_mod, i32_to_bytes,
    u32_from_bytes, u32_mod, u32_to_bytes,
  },
  Infra,
};

use super::utils::{
  approx_equal, f32_add, f32_div, f32_mul, f32_sub, i32_add, i32_div,
  i32_mul, i32_sub, u32_add, u32_div, u32_mul, u32_sub,
};

pub const OPERATIONS: [[fn(param1: &mut TypedByte, param2: [u8; 4]);
  3]; 5] = [
  [
    |v1, v2| v1.value = u32_to_bytes(u32_add(**v1, v2)),
    |v1, v2| v1.value = i32_to_bytes(i32_add(**v1, v2)),
    |v1, v2| v1.value = f32_to_bytes(f32_add(**v1, v2)),
  ],
  [
    |v1, v2| {
      v1.value = u32_to_bytes({
        if **v1 >= v2 {
          u32_sub(**v1, v2)
        } else {
          0u32
        }
      })
    },
    |v1, v2| v1.value = i32_to_bytes(i32_sub(**v1, v2)),
    |v1, v2| v1.value = f32_to_bytes(f32_sub(**v1, v2)),
  ],
  [
    |v1, v2| v1.value = u32_to_bytes(u32_mul(**v1, v2)),
    |v1, v2| v1.value = i32_to_bytes(i32_mul(**v1, v2)),
    |v1, v2| v1.value = f32_to_bytes(f32_mul(**v1, v2)),
  ],
  [
    |v1, v2| v1.value = u32_to_bytes(u32_div(**v1, v2)),
    |v1, v2| v1.value = i32_to_bytes(i32_div(**v1, v2)),
    |v1, v2| v1.value = f32_to_bytes(f32_div(**v1, v2)),
  ],
  [
    |v1, v2| v1.value = u32_to_bytes(u32_mod(**v1, v2)),
    |v1, v2| v1.value = i32_to_bytes(i32_mod(**v1, v2)),
    |v1, v2| v1.value = f32_to_bytes(f32_mod(**v1, v2)),
  ],
];

pub struct Interpreter<'a, const A: usize> {
  pub memory: [TypedByte; A],
  pub debug: bool,
  pub stack: Stack<65535>,
  convertions: [fn(&mut TypedByte, &mut Interpreter<A>) -> bool; 4],
  operations: [fn(&mut Interpreter<A>, &mut dyn Infra); 256],
  time: Option<Instant>,
  duration: Option<Duration>,
  index: usize,
  instruction: Instruction,
  buffer: &'a [u8],
  err: Option<IError>,
}
#[inline]
fn not_convert<const A: usize>(
  _: &mut TypedByte,
  _interpreter: &mut Interpreter<A>,
) -> bool {
  true
}

#[inline]
fn convert_chupou<'a, const A: usize>(
  byte: &mut TypedByte,
  interpreter: &mut Interpreter<A>,
) -> bool {
  *byte = interpreter.stack.top();
  interpreter.stack.pop();
  true
}

#[inline]
fn conver_fudeu<'a, const A: usize>(
  byte: &mut TypedByte,
  interpreter: &mut Interpreter<A>,
) -> bool {
  let address = byte.force_u32();

  *byte = interpreter.memory[address as usize % A];
  true
}

#[inline]
fn convert_penetrou<'a, const A: usize>(
  byte: &mut TypedByte,
  interpreter: &mut Interpreter<A>,
) -> bool {
  let a = interpreter.stack.top();
  let address = a.force_u32();
  interpreter.stack.pop();
  *byte = interpreter.memory[address as usize % A];
  true
}

fn dns<const A: usize>(int: &mut Interpreter<A>, _: &mut dyn Infra) {
  int.err =
    someierror!("Operation is not operating, please suck my dick");
  return;
}
// OPCODE 1
#[inline]
fn operations<const A: usize>(
  interpreter: &mut Interpreter<A>,
  _: &mut dyn Infra, // [mut param1, mut param2, mut param3]: [(TypedByte, usize, bool); 3],
) {
  let mut param1 = interpreter.instruction.params[0];
  let mut param2 = interpreter.instruction.params[1];
  let mut param3 = interpreter.instruction.params[2];

  convert!(interpreter, param1);
  convert!(interpreter, param2);
  convert!(interpreter, param3);

  let kind = force_u32!(interpreter, param1.0);

  if param2.2 {
    param2.0.convert(param3.0.r#type);
  }

  if param3.2 {
    param3.0.convert(param2.0.r#type);
  }

  OPERATIONS[kind as usize][param2.0.r#type as usize](
    &mut param2.0,
    param3.0.value,
  );

  interpreter.stack.push(param2.0);
}

//OPCODE 2
#[inline]
fn if_lower<const A: usize>(
  interpreter: &mut Interpreter<A>,
  _: &mut dyn Infra,
) {
  let mut param1 = interpreter.instruction.params[0];
  let mut param2 = interpreter.instruction.params[1];
  let mut param3 = interpreter.instruction.params[2];
  convert!(interpreter, param1);
  convert!(interpreter, param2);
  convert!(interpreter, param3);

  let pos = force_u32!(interpreter, param3.0);

  match param1.0.r#type {
    Type::Usigned => {
      if u32_from_bytes(*param1.0) < u32_from_bytes(*param2.0) {
        interpreter.index = (pos * 15) as usize;
      }
    }
    Type::Signed => {
      if i32_from_bytes(*param1.0) < i32_from_bytes(*param2.0) {
        interpreter.index = (pos * 15) as usize;
      }
    }
    Type::Floating => {
      if i32_from_bytes(*param1.0) < i32_from_bytes(*param2.0) {
        interpreter.index = (pos * 15) as usize;
      }
    }
  }
}

//OPCODE 3
#[inline]
fn push_stack<const A: usize>(
  interpreter: &mut Interpreter<A>,
  _: &mut dyn Infra,
) {
  let mut param1 = interpreter.instruction.params[0];
  convert!(interpreter, param1);
  interpreter.stack.push(param1.0);
}

//OPCODE 4
#[inline]
fn if_true_jump<const A: usize>(
  interpreter: &mut Interpreter<A>,
  _: &mut dyn Infra,
) {
  let mut param1 = interpreter.instruction.params[0];
  let mut param2 = interpreter.instruction.params[1];
  let mut param3 = interpreter.instruction.params[2];

  convert!(interpreter, param1);
  convert!(interpreter, param2);
  convert!(interpreter, param3);
  let pos = force_u32!(interpreter, param3.0);

  if param1.0.r#type != param2.0.r#type {
    interpreter.err = someierror!("Comparing incompatible types");
    return;
  }

  if param1.0.r#type == Type::Floating {
    let value = f32_from_bytes(*param1.0);
    let value2 = f32_from_bytes(*param2.0);
    if approx_equal(value, value2, 4) {
      interpreter.index = (pos * 15) as usize;
    }
  } else if *param1.0 == *param2.0 {
    interpreter.index = (pos * 15) as usize;
  };
}

//OPCODE 5
#[inline(always)]
fn vstack_jump<const A: usize>(
  interpreter: &mut Interpreter<A>,
  _: &mut dyn Infra,
) {
  let mut param1 = interpreter.instruction.params[0];

  convert!(interpreter, param1);
  let value = force_u32!(interpreter, param1.0);
  interpreter.index =
    ((value * 15) * !interpreter.stack.is_empty() as u32) as usize;
}

//OPCODE 6
#[inline(always)]
fn input<const A: usize>(
  interpreter: &mut Interpreter<A>,
  infra: &mut dyn Infra,
) {
  let mut param1 = interpreter.instruction.params[0];
  let mut param2 = interpreter.instruction.params[1];
  let mut param3 = interpreter.instruction.params[2];

  convert!(interpreter, param1);
  convert!(interpreter, param2);
  convert!(interpreter, param3);

  let mut value = force_u32!(interpreter, param1.0) as usize;
  let kind = force_u32!(interpreter, param2.0);
  let limit = force_u32!(interpreter, param3.0) as usize;

  try_err!(interpreter, std::io::stdout().flush());
  let buff = try_err!(interpreter, infra.read_line());

  match kind {
    1 => {
      match buff.trim().parse::<u32>() {
        Ok(x) => interpreter.memory[value % A] = x.into(),
        Err(_) => interpreter.stack.push(1u32.into()),
      };
    }
    2 => match buff.trim().parse::<i32>() {
      Ok(x) => interpreter.memory[value % A] = x.into(),
      Err(_) => interpreter.stack.push(1u32.into()),
    },
    3 => match buff.trim().parse::<f32>() {
      Ok(x) => interpreter.memory[value % A] = x.into(),
      Err(_) => interpreter.stack.push(1u32.into()),
    },
    _ => {
      for (i, char) in buff.chars().enumerate() {
        if i < limit {
          let char = if char == '\n' || char == '\0' {
            interpreter.memory[value % A] = [0; 4].into();
            continue;
          } else {
            char
          };
          let mut bytes: [u8; 4] = [0, 0, 0, 0];

          char.encode_utf8(&mut bytes);

          interpreter.memory[value % A] =
            (u32_from_bytes(bytes)).into();
          value += 1;
        } else {
          break;
        }
      }
      interpreter.memory[value % A] = 0u32.into();
    }
  };
}

//OPCODE 7
#[inline(always)]
fn print<const A: usize>(
  interpreter: &mut Interpreter<A>,
  infra: &mut dyn Infra,
) {
  if interpreter.stack.sx == 0 {
    interpreter.err = someierror!(
      "Trying to use the stack while empty in command {}",
      interpreter.current_command()
    );
    return;
  }
  let stack_byte = top!(interpreter, interpreter.stack);

  infra.print(&*stack_byte);

  interpreter.stack.pop();
}

//OPCODE 8
#[inline(always)]
fn printnumber<const A: usize>(
  interpreter: &mut Interpreter<A>,
  infra: &mut dyn Infra,
) {
  let TypedByte { value, r#type } =
    top!(interpreter, interpreter.stack);

  match r#type {
    Type::Floating => {
      infra.print(f32_from_bytes(value).to_string().as_bytes())
    }
    Type::Signed => {
      infra.print(i32_from_bytes(value).to_string().as_bytes())
    }
    Type::Usigned => {
      infra.print(u32_from_bytes(value).to_string().as_bytes())
    }
  }

  interpreter.stack.pop();
}

//OPCODE 9
#[inline(always)]
fn jump<const A: usize>(
  interpreter: &mut Interpreter<A>,
  _: &mut dyn Infra,
) {
  let mut param1 = interpreter.instruction.params[0];

  convert!(interpreter, param1);
  let value = force_u32!(interpreter, param1.0);
  interpreter.index = (value * 15) as usize;
}

//OPCODE 10
#[inline(always)]
fn set<const A: usize>(
  interpreter: &mut Interpreter<A>,
  _: &mut dyn Infra,
) {
  let mut param1 = interpreter.instruction.params[0];

  convert!(interpreter, param1);

  let address = force_u32!(interpreter, param1.0) as usize;
  let typed_byte = top!(interpreter, interpreter.stack);
  interpreter.stack.pop();
  interpreter.memory[address % A] = typed_byte;
}

//OPCODE 11
#[inline(always)]
fn pop_stack<const A: usize>(
  interpreter: &mut Interpreter<A>,
  _: &mut dyn Infra,
) {
  interpreter.stack.pop();
}

//OPCODE 12
#[inline(always)]
fn load_string<const A: usize>(
  interpreter: &mut Interpreter<A>,
  _: &mut dyn Infra,
) {
  let mut param1 = interpreter.instruction.params[0];

  convert!(interpreter, param1);
  let mut value = force_u32!(interpreter, param1.0) as usize;
  let mut buffer: Vec<u32> = Vec::new();
  loop {
    if value == interpreter.memory.len() {
      break;
    }
    let temp = u32_from_bytes(*interpreter.memory[value % A]);
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
      value: u32_to_bytes(i),
      r#type: Type::Usigned,
    });
  }
}

//OPCODE 13
#[inline(always)]
fn time_operations<const A: usize>(
  interpreter: &mut Interpreter<A>,
  _: &mut dyn Infra,
) {
  let mut param1 = interpreter.instruction.params[0];

  convert!(interpreter, param1);
  match force_u32!(interpreter, param1.0) {
    // SET ax
    0 => {
      interpreter.time = Some(Instant::now());
    }
    //SET bx
    1 => {
      let a = top!(interpreter, interpreter.stack).value;
      interpreter.stack.pop();
      let b = top!(interpreter, interpreter.stack).value;
      let result = [b[0], b[1], b[2], b[3], a[0], a[1], a[2], a[3]];

      interpreter.duration = Some(Duration::from_millis(unsafe {
        std::mem::transmute(result)
      }));
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
}
//OPCODE 14
#[inline(always)]
fn flush<const A: usize>(
  _: &mut Interpreter<A>,
  infra: &mut dyn Infra,
) {
  infra.flush();
}
//OPCODE 15
#[inline(always)]
fn terminal_commands<const A: usize>(
  interpreter: &mut Interpreter<A>,
  infra: &mut dyn Infra,
) {
  let mut param1 = interpreter.instruction.params[0];

  convert!(interpreter, param1);

  match force_u32!(interpreter, param1.0) {
    // RAW MODE
    0 => {
      let on_off =
        force_u32!(interpreter, top!(interpreter, interpreter.stack));
      interpreter.stack.pop();
      if on_off == 0 {
        try_err!(interpreter, infra.disable_raw_mode());
      } else {
        try_err!(interpreter, infra.enable_raw_mode());
      }
    }
    // CLEAR
    1 => {
      let r#type =
        force_u32!(interpreter, top!(interpreter, interpreter.stack));
      interpreter.stack.pop();
      if r#type == 0 {
        try_err!(interpreter, infra.clear_purge());
      } else {
        try_err!(interpreter, infra.clear_all());
      }
    }
    // POLL KEYBOARD
    2 => {
      let a = *top!(interpreter, interpreter.stack);
      interpreter.stack.pop();
      let b = *top!(interpreter, interpreter.stack);
      interpreter.stack.pop();
      let result = [b[0], b[1], b[2], b[3], a[0], a[1], a[2], a[3]];
      let value = try_err!(
        interpreter,
        infra.poll(unsafe {
          std::mem::transmute::<[u8; 8], u64>(result)
        })
      );
      interpreter.stack.push(value.into());
    }
    // SHOW/HIDE CURSOR
    3 => {
      let on_off =
        force_u32!(interpreter, top!(interpreter, interpreter.stack));
      interpreter.stack.pop();
      if on_off == 0 {
        try_err!(interpreter, infra.hide_cursor());
      } else {
        try_err!(interpreter, infra.show_cursor());
      }
    }
    // MOVE CURSOR
    4 => {
      let x =
        force_u32!(interpreter, top!(interpreter, interpreter.stack));
      interpreter.stack.pop();
      let y =
        force_u32!(interpreter, top!(interpreter, interpreter.stack));
      interpreter.stack.pop();
      try_err!(interpreter, infra.move_cursor(x, y));
    }
    // FONT COLOR
    5 => {
      let color =
        force_u32!(interpreter, top!(interpreter, interpreter.stack));
      interpreter.stack.pop();
      try_err!(interpreter, infra.use_color(color));
    }
    //BACKGROUND
    6 => {
      let color =
        force_u32!(interpreter, top!(interpreter, interpreter.stack));
      interpreter.stack.pop();
      try_err!(interpreter, infra.use_background(color));
    }
    _ => {
      interpreter.err = someierror!("Invalid terminal command");
      return;
    }
  }
}

// OPCODE 16
#[inline(always)]
fn random<const A: usize>(
  interpreter: &mut Interpreter<A>,
  _: &mut dyn Infra,
) {
  let mut rng = rand::thread_rng();
  interpreter.stack.push(rng.gen::<f32>().into());
}

impl<'a, const A: usize> Interpreter<'a, A> {
  pub const fn new(buffer: &'a [u8]) -> Self {
    Self {
      memory: [TypedByte {
        r#type: Type::Usigned,
        value: [0; 4],
      }; A],
      stack: Stack::new(),
      debug: false,
      time: None,
      duration: None,
      convertions: [
        not_convert,
        convert_chupou,
        conver_fudeu,
        convert_penetrou,
      ],
      operations: [
        dns,
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
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
        dns,
      ],
      instruction: Instruction::new(),
      index: 0,
      err: None,
      buffer,
    }
  }

  pub fn current_command(&self) -> usize {
    self.index / 15
  }

  pub fn get_bytes_data(buffer: [u8; 15], inst: &mut Instruction) {
    let converted_types = convert_kind_byte(buffer[14]);
    let converted_kind = convert_kind_byte(buffer[1]);

    let [param1_convert, param2_convert] =
      convert_two_bits(converted_kind[3] as u8);
    let [param3_convert, _] =
      convert_two_bits(converted_types[3] as u8);

    inst.opcode = buffer[0];

    inst.params[0].0 = TypedByte {
      value: [buffer[2], buffer[3], buffer[4], buffer[5]],
      r#type: Type::from(converted_types[0]),
    };
    inst.params[0].2 = param1_convert;
    inst.params[0].1 = converted_kind[0];

    inst.params[1].0 = TypedByte {
      value: [buffer[6], buffer[7], buffer[8], buffer[9]],
      r#type: Type::from(converted_types[1]),
    };
    inst.params[1].2 = param2_convert;
    inst.params[1].1 = converted_kind[1];

    inst.params[2].0 = TypedByte {
      value: [buffer[10], buffer[11], buffer[12], buffer[13]],
      r#type: Type::from(converted_types[2]),
    };
    inst.params[2].2 = param3_convert;
    inst.params[2].1 = converted_kind[2];
  }
  pub fn debug(&mut self, new: bool) {
    self.debug = new;
  }

  pub fn run_buffer(
    &mut self,
    infra: &mut dyn Infra,
  ) -> Result<(), IError> {
    loop {
      if self.index + 15 > self.buffer.len() {
        break;
      }
      let converted_types =
        convert_kind_byte(self.buffer[self.index + 14]);
      let converted_kind =
        convert_kind_byte(self.buffer[self.index + 1]);

      let [param1_convert, param2_convert] =
        convert_two_bits(converted_kind[3] as u8);
      let [param3_convert, _] =
        convert_two_bits(converted_types[3] as u8);

      self.instruction.opcode = self.buffer[self.index];

      self.instruction.params[0].0 = TypedByte {
        value: [
          self.buffer[self.index + 2],
          self.buffer[self.index + 3],
          self.buffer[self.index + 4],
          self.buffer[self.index + 5],
        ],
        r#type: Type::from(converted_types[0]),
      };
      self.instruction.params[0].2 = param1_convert;
      self.instruction.params[0].1 = converted_kind[0];

      self.instruction.params[1].0 = TypedByte {
        value: [
          self.buffer[self.index + 6],
          self.buffer[self.index + 7],
          self.buffer[self.index + 8],
          self.buffer[self.index + 9],
        ],
        r#type: Type::from(converted_types[1]),
      };
      self.instruction.params[1].2 = param2_convert;
      self.instruction.params[1].1 = converted_kind[1];

      self.instruction.params[2].0 = TypedByte {
        value: [
          self.buffer[self.index + 10],
          self.buffer[self.index + 11],
          self.buffer[self.index + 12],
          self.buffer[self.index + 13],
        ],
        r#type: Type::from(converted_types[2]),
      };
      self.instruction.params[2].2 = param3_convert;
      self.instruction.params[2].1 = converted_kind[2];

      self.index += 15;
      self.execute_command(infra);

      if let Some(err) = &self.err {
        return Err(err.clone());
      }
    }
    Ok(())
  }
  #[inline]
  pub fn convert(
    &mut self,
    byte: &mut TypedByte,
    r#type: usize,
  ) -> bool {
    self.convertions[r#type](byte, self)
  }
  #[inline]
  pub fn execute_command(&mut self, infra: &mut dyn Infra) {
    self.operations[self.instruction.opcode as usize](self, infra);
  }
}
