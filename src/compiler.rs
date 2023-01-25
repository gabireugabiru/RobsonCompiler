use std::{
  collections::HashMap,
  io::{BufRead, BufReader},
};

use crate::{
  data_struct::{IError, TypedByte},
  utils::{self, create_kind_byte, create_two_bits},
  CompilerInfra, ROBSON_FOLDER, STDRB_FOLDER,
};

pub struct Compiler {
  lines: Vec<String>,
  opcode_params: [u8; 17],
  names: HashMap<String, usize>,
  files: HashMap<String, usize>,
  pos: usize,
  debug: bool,
  current_command: usize,
  buffer: Vec<u8>,
  infra: Box<dyn CompilerInfra>,
  is_preload: bool,
  compiled_stack: Vec<String>,
  last_opcode: u8,
  offset: usize,
  inner: usize,
  path: String,
}
impl Compiler {
  pub fn new(
    mut path: String,
    infra: Box<dyn CompilerInfra>,
  ) -> Result<Self, IError> {
    if path.contains("stdrb/") {
      path = path.replace("stdrb/", "");

      let home = utils::home_dir()
        .ok_or_else(|| IError::message("Couldnt find Home Path"))?;
      let mut new_path = home.join(ROBSON_FOLDER).join(STDRB_FOLDER);

      for join in path.split("/") {
        new_path = new_path.join(join);
      }
      path = new_path
        .to_str()
        .ok_or(IError::message(&format!(
          "Failed to parse the path stdrb/{}",
          path
        )))?
        .to_string();
    }

    let file =
      std::fs::File::options().read(true).open(path.clone())?;
    let buff_reader = BufReader::new(&file);
    let lines = buff_reader
      .lines()
      .flat_map(|a| a.ok())
      .collect::<Vec<String>>();
    Ok(Self {
      buffer: Vec::new(),
      debug: false,
      infra,
      last_opcode: 0,
      offset: 0,
      files: HashMap::new(),
      is_preload: false,
      lines,
      current_command: 0,
      names: HashMap::new(),
      compiled_stack: Vec::new(),
      opcode_params: [
        0, 3, 3, 1, 3, 1, 3, 0, 0, 1, 1, 0, 1, 1, 0, 1, 0,
      ],
      pos: 0,
      path,
      inner: 0,
    })
  }
  pub fn inner_in(&mut self, current: usize) {
    self.inner = current + 1;
  }
  pub fn set_files(
    &mut self,
    new_compiled_files: HashMap<String, usize>,
  ) {
    self.files = new_compiled_files;
  }
  pub fn compiled_stack(
    &mut self,
    current: Vec<String>,
    new_path: &str,
  ) -> Result<(), IError> {
    self.compiled_stack = current;
    if self.compiled_stack.contains(&new_path.to_owned()) {
      return Err(IError::message("Creating infinite compilation"));
    } else {
      self.compiled_stack.push(new_path.to_owned());
      Ok(())
    }
  }
  pub fn set_offset(&mut self, offset: usize) {
    self.offset = offset;
  }
  pub fn set_preload(&mut self, new_preload: bool) {
    self.is_preload = new_preload;
  }
  pub fn compile(&mut self) -> Result<Vec<u8>, IError> {
    if let Some(err) = self.start_command_alias() {
      return Err(err);
    }
    loop {
      if self.verify_index_overflow(self.pos) {
        break;
      }
      let pre_string = self.lines[self.pos].to_owned();
      let mut string = pre_string.as_str();

      string = Self::remove_comments(string).trim();

      // skip aliases
      if string.ends_with(':') {
        self.pos += 1;
        continue;
      }

      // //skip spaces
      if string.is_empty() {
        self.pos += 1;
        continue;
      }

      // implement multiple module instructions
      if string.contains("robsons") {
        let splited: Vec<&str> = string.split(' ').collect();
        if splited.len() != 2 {
          return Err(IError::message("Malformated robsons"));
        }
        let file_path = splited[1];
        let mut inner_spaces = String::from("");
        if self.inner > 0 {
          if self.inner > 1 {
            for _i in 0..self.inner {
              inner_spaces.push_str("  ");
            }
          }
          inner_spaces.push_str(" +->");
        }
        if !self.is_preload {
          self.infra.color_print(inner_spaces, 40);
          self.infra.color_print(
            format!(" Compiling {file_path}\n"),
            match self.inner {
              0 => 2,
              _ => 2,
            },
          );
        }
        let mut compiler = match Compiler::new(
          file_path.to_owned(),
          self.infra.clone_self(),
        ) {
          Ok(a) => a,
          Err(err) => {
            if err.to_string().contains("os error 2") {
              return Err(IError::message(format!(
                "No such file '{}' (os error 2)",
                file_path
              )));
            } else {
              return Err(err);
            }
          }
        };
        compiler.set_files(self.files.clone());
        compiler.set_preload(self.is_preload);
        compiler.inner_in(self.inner);
        compiler.set_offset(self.current_command + self.offset);

        let buffer = compiler.compile()?;
        self.current_command += buffer.len() / 15;
        for i in buffer {
          self.buffer.push(i);
        }
        self.last_opcode = 0;
        self.pos += 1;
        continue;
      }

      // Implements the push abreviation
      if self.last_opcode == 3 && !string.contains("robson") {
        self.push_command(
          3,
          [string.to_owned(), "".to_owned(), "".to_owned()],
        )?;
        self.pos += 1;
        continue;
      }

      //get params and opcodes
      let mut opcode: u8 = 0;
      let mut params: [String; 3] =
        ["".to_owned(), "".to_owned(), "".to_owned()];

      let spaces: Vec<&str> = string.split(' ').collect();

      for i in spaces {
        if i != "robson" {
          return Err(IError::message(format!(
            "Invalid token for opcode in line {}, '{}'",
            self.pos + 1,
            i
          )));
        }
        opcode += 1;
      }
      if opcode as usize >= self.opcode_params.len() {
        return Err(IError::message(format!(
          "invalid opcode of line {}",
          self.pos + 1
        )));
      }
      let param_count = self.opcode_params[opcode as usize];
      for i in 0..param_count {
        self.pos += 1;
        if self.verify_index_overflow(self.pos) {
          return Err(IError::message(format!(
            "missing params command of line {}",
            self.pos - i as usize,
          )));
        }
        let string = self.lines[self.pos].to_owned();
        if string.trim().len() < 2 {
          return Err(IError::message(format!(
            "missing params command of line {}",
            self.pos - i as usize,
          )));
        }
        params[i as usize] = string;
      }

      //update and run command
      self.pos += 1;

      self.push_command(opcode, params)?;

      self.last_opcode = opcode;
    }
    Ok(self.buffer.clone())
  }

  pub fn get_cached_robsons_size(
    &mut self,
    path: &str,
    command_number: usize,
  ) -> Result<usize, IError> {
    match self.files.get(path) {
      Some(a) => Ok(*a),
      None => {
        // compile file and cache it
        let mut compiler = match Compiler::new(
          path.to_owned(),
          self.infra.clone_self(),
        ) {
          Ok(a) => a,
          Err(err) => {
            if err.to_string().contains("os error 2") {
              return Err(IError::message(format!(
                "No such file '{}' (os error 2)",
                path
              )));
            } else {
              return Err(err);
            }
          }
        };
        self.infra.color_print(format!("Preloading {path}\n"), 14);
        compiler.set_offset(command_number);
        compiler.set_preload(true);
        compiler.inner_in(self.inner);
        compiler.set_files(self.files.clone());
        if let Err(err) = compiler
          .compiled_stack(self.compiled_stack.clone(), &self.path)
        {
          return Err(err);
        }

        match compiler.compile() {
          Ok(_) => {
            // inherit the compiled files
            self.files = compiler.files.clone();
            self
              .files
              .insert(path.to_owned(), compiler.current_command);
            Ok(compiler.current_command)
          }
          Err(err) => return Err(err),
        }
      }
    }
  }

  pub fn start_command_alias(&mut self) -> Option<IError> {
    struct LastCommand {
      value: u8,
      pos: usize,
    }
    let mut command_number = 0;
    let mut last_command = LastCommand { pos: 0, value: 0 };
    for (pos, i) in self.lines.clone().iter().enumerate() {
      let string = Self::remove_comments(i).trim().to_owned();
      if string.is_empty() {
        continue;
      }
      //add alias if it is an alias
      if string.ends_with(':') {
        let value = string.trim().replace(':', "");
        if self.names.get(&value).is_some() {
          return Some(IError::message(format!(
            "duplicated alias: {}",
            value
          )));
        }
        if self.debug {
          self.infra.println(format!("{}: {}", value, pos + 1));
        }
        self.names.insert(value, command_number + self.offset);
      } else {
        //if is not an check what it is
        if string.starts_with("robsons") {
          //if is an include compile the include to get the correct value of the aliases
          let splited: Vec<&str> = string.split(' ').collect();
          if splited.len() != 2 {
            return Some(IError::message("malformated robsons"));
          }
          let path = splited[1];

          // get offset from cache if possible
          let new_offset = match self
            .get_cached_robsons_size(path, command_number)
          {
            Ok(a) => a,
            Err(err) => return Some(err),
          };

          command_number += new_offset;
        } else if string.starts_with("robson") {
          // if is a command just add it
          command_number += 1;
          let mut opcode: u8 = 0;
          let spaces: Vec<&str> = string.split(' ').collect();

          for i in spaces {
            if i != "robson" {
              return Some(IError::message(format!(
                "invalid token for opcode in line {}, '{}'",
                pos + 1,
                i
              )));
            }
            opcode += 1;
          }
          last_command.value = opcode;
          last_command.pos = pos;
        } else if last_command.value == 3
          && last_command.pos + 1 != pos
        {
          command_number += 1;
        }
      }
    }
    None
  }
  pub fn remove_comments(string: &str) -> &str {
    let mut res = string;

    let comments = string.split(';').collect::<Vec<&str>>();
    if !comments.is_empty() {
      res = comments[0].trim();
    }
    res
  }

  fn verify_index_overflow(&self, pos: usize) -> bool {
    self.lines.len() <= pos
  }
  pub fn push_command(
    &mut self,
    opcode: u8,
    params: [String; 3],
  ) -> Result<(), IError> {
    self.buffer.push(opcode);

    let (param1, param1_kind, param1_types, param1_convert) =
      self.get_kind_value(params[0].trim())?;
    let param1 = param1.value;

    let (param2, param2_kind, param2_types, param2_convert) =
      self.get_kind_value(params[1].trim())?;
    let param2 = param2.value;

    let (param3, param3_kind, param3_types, param3_convert) =
      self.get_kind_value(params[2].trim())?;
    let param3 = param3.value;

    self.buffer.push(utils::create_kind_byte(
      param1_kind,
      param2_kind,
      param3_kind,
      create_two_bits([param1_convert, param2_convert]),
    ));

    for i in param1 {
      self.buffer.push(i);
    }
    for i in param2 {
      self.buffer.push(i);
    }
    for i in param3 {
      self.buffer.push(i);
    }
    self.buffer.push(create_kind_byte(
      param1_types,
      param2_types,
      param3_types,
      create_two_bits([param3_convert, false]),
    ));
    self.current_command += 1;
    Ok(())
  }
  pub fn get_kind_value(
    &self,
    parameter: &str,
  ) -> Result<(TypedByte, u8, u8, bool), IError> {
    if parameter.is_empty() {
      return Ok((0u32.into(), 0, 0, false));
    }
    let splited: Vec<&str> = parameter.split(' ').collect();

    if splited.len() < 2 {
      return Err(IError::message(format!(
        "Malformated param at line {}",
        self.pos
      )));
    }
    let mut convert = false;
    if splited.len() == 3 {
      if splited[2] == "robson" {
        convert = true;
      } else {
        return Err(IError::message(format!(
          "Malformated param at line {}, expected 'robson'",
          self.pos
        )));
      }
    }

    match splited[0] {
      "comeu" => {
        let mut value = splited[1].trim().to_owned();
        let first = value.chars().collect::<Vec<char>>()[0];
        match first {
          'f' => {
            value = value.replace('f', "");
            Ok((value.parse::<f32>()?.into(), 0, 2, convert))
          }
          'i' => {
            value = value.replace('i', "");
            Ok((value.parse::<i32>()?.into(), 0, 1, convert))
          }
          _ => Ok((
            splited[1].trim().parse::<u32>()?.into(),
            0,
            0,
            convert,
          )),
        }
      }
      "chupou" => {
        let value = splited[1].parse::<u32>()?;
        Ok((value.into(), 1, 0, convert))
      }
      "fudeu" => {
        let value = splited[1].trim().parse::<u32>()?;
        Ok((value.into(), 2, 0, convert))
      }
      "lambeu" => {
        let value = splited[1].trim();
        if value.chars().collect::<Vec<char>>()[0] != ':' {
          return Err(IError::message(format!(
            "malformated name in command at {}, '{}'",
            self.pos, value
          )));
        }
        let value = value.replace(':', "");

        let a = self.names.get(&value).ok_or_else(|| {
          IError::message(format!("cant find {}", value))
        })?;
        Ok(((*a as u32).into(), 0, 0, convert))
      }
      "penetrou" => {
        let value = splited[1].trim().parse::<u32>()?;
        Ok((value.into(), 3, 0, convert))
      }
      token => {
        return Err(IError::message(format!(
          "Unexpect token for param at line {}, '{}'",
          self.pos, token
        )))
      }
    }
  }
}
