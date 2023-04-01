use std::{collections::HashMap, path::PathBuf, str::FromStr};

use crate::{
  data_struct::{IError, Stack, Type, TypedByte},
  macros::{compiler, ierror, replace_params, sanitize_param},
  utils::{self, create_kind_byte, create_two_bits},
  CompilerInfra, ROBSON_FOLDER, STDRB_FOLDER,
};

pub struct Compiler {
  lines: Vec<String>,
  opcode_params: [u8; 17],
  names: HashMap<String, usize>,
  files: HashMap<String, (usize, usize)>,
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
  is_static: bool,
  macro_params: Option<HashMap<String, String>>,
  macro_current: Stack<10>,
  macro_jump: Stack<10>,
}
impl Compiler {
  pub fn new<'a>(
    mut path: String,
    infra: Box<dyn CompilerInfra>,
  ) -> Result<Self, IError> {
    if path.contains("stdrb/") {
      path = path.replace("stdrb/", "");

      let home = infra
        .home_dir()
        .ok_or_else(|| IError::message("Couldnt find Home Path"))?;
      let mut new_path = PathBuf::from_str(&home)
        .ok()
        .ok_or(IError::message("Failed to parse home path"))?;

      new_path = new_path.join(ROBSON_FOLDER).join(STDRB_FOLDER);

      for join in path.split('/') {
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
    let lines = infra.lines(&path)?;
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
      macro_jump: Stack::new(),
      macro_current: Stack::new(),
      is_static: true,
      opcode_params: [
        0, 3, 3, 1, 3, 1, 3, 0, 0, 1, 1, 0, 1, 1, 0, 1, 0,
      ],
      pos: 0,
      path,
      inner: 0,
      macro_params: None,
    })
  }

  pub fn get_file_params(&self) -> Result<Vec<u32>, IError> {
    if !self.lines.is_empty() {
      let mut is_error = false;
      let splited: Vec<u32> = self.lines[0]
        .split("$ROBSON")
        .flat_map(|mut a| {
          a = a.trim();
          if a.is_empty() {
            None
          } else {
            let b = a.parse::<u32>();
            if b.is_err() {
              is_error = true;
            }
            b.ok()
          }
        })
        .collect();
      if is_error {
        return ierror!(
          "Malformated param requirement at '{}' in '{}'",
          self.lines[0],
          self.path
        );
      }
      Ok(splited)
    } else {
      ierror!("The file '{}' is empty", self.path)
    }
  }

  pub fn inner_in(&mut self, current: usize) {
    self.inner = current + 1;
  }
  pub fn set_files(
    &mut self,
    new_compiled_files: HashMap<String, (usize, usize)>,
  ) {
    self.files = new_compiled_files;
  }

  pub fn set_macro_params(
    &mut self,
    params: HashMap<String, String>,
  ) {
    self.macro_params = Some(params);
  }
  pub fn compiled_stack(
    &mut self,
    current: Vec<String>,
    new_path: &str,
  ) -> Result<(), IError> {
    self.compiled_stack = current;
    if self.compiled_stack.contains(&new_path.to_owned()) {
      ierror!("Creating infinite compilation")
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
    self.start_command_alias()?;

    if self.macro_params.is_some() {
      self.pos += 1;
    }

    loop {
      if self.verify_index_overflow(self.pos) {
        break;
      }
      let mut string = self.lines[self.pos].clone();

      string = Self::remove_comments(&string).trim().to_owned();

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

      replace_params!(self, string);

      if string == "SEMPRE#ROBSON" {
        self.is_static = false;
        if self.macro_jump.sx != 0 {
          // if let Some(top) = self.macro_jump.top() {
          self.pos = self.macro_jump.top().into();
          continue;
        } else {
          self.pos += 1;
          continue;
        }
      }
      if string == "PARE#ROBSON" {
        self.macro_current.pop();
        self.pos += 1;
        continue;
      }

      //INCLUDE LOGIC
      if string.starts_with("robsons") {
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
        let mut compiler = compiler!(file_path, self.infra);

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
      // MACRO LOGIC
      } else if string.contains("robsons") {
        let split: Vec<&str> = string.split('[').collect();
        let split2: Vec<&str> = string.split(']').collect();

        if split.len() == 2 && split2.len() == 2 {
          let inside: Vec<&str> =
            split2[0][1..].split("robsons").collect();

          if inside.len() != 2 {
            return ierror!("Malformated robsons macro at {string}");
          }
          if inside[1].trim().is_empty() {
            return ierror!("Malformated robsons macro at {string}");
          }

          let mut compiler = compiler!(inside[1].trim(), self.infra);

          let params_count = compiler.get_file_params()?;

          let mut params: HashMap<String, String> = HashMap::new();
          for i in params_count {
            self.pos += 1;
            if self.verify_index_overflow(self.pos) {
              return ierror!(
                "Missing params command of line {}",
                self.pos - i as usize,
              );
            }

            let mut string = self.lines[self.pos].to_owned();

            replace_params!(self, string);

            sanitize_param!(self, string);

            if string.trim().is_empty() {
              return ierror!(
                "Missing params command of line {}",
                self.pos - i as usize,
              );
            }
            let update = params.insert(format!("{i}$ROBSON"), string);

            if update.is_some() {
              return ierror!("Duplicated param at in {}", self.path);
            }
          }

          compiler.set_macro_params(params);
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
          return ierror!(
            "Invalid token for opcode in line {}, '{}'",
            self.pos + 1,
            i
          );
        }
        opcode += 1;
      }
      if opcode as usize >= self.opcode_params.len() {
        return ierror!("Invalid opcode of line {}", self.pos + 1);
      }
      let param_count = self.opcode_params[opcode as usize];
      for i in 0..param_count {
        self.pos += 1;
        if self.verify_index_overflow(self.pos) {
          return ierror!(
            "Missing params command of line {}",
            self.pos - i as usize,
          );
        }
        let mut string = self.lines[self.pos].to_owned();

        replace_params!(self, string);

        if string.trim().is_empty() {
          return ierror!(
            "Missing params command of line {}",
            self.pos - i as usize,
          );
        }
        params[i as usize] = string;
      }

      //update and compile command
      self.pos += 1;

      self.push_command(opcode, params)?;

      self.last_opcode = opcode;
    }
    Ok(self.buffer.clone())
  }

  pub fn get_cached_macro_size(
    &mut self,
    path: &str,
    command_number: usize,
    mut pos: usize,
  ) -> Result<(usize, usize), IError> {
    match self.files.get(path) {
      Some(a) => Ok(*a),
      None => {
        let mut compiler = compiler!(path, self.infra);

        let params_count = compiler.get_file_params()?;
        let mut params = HashMap::new();

        pos += 1;

        for i in &params_count {
          if self.verify_index_overflow(pos) {
            return ierror!("Missing param in line {}", pos);
          }
          let tmp = self.is_preload;
          self.is_preload = true;
          let mut line = self.lines[pos].clone();

          sanitize_param!(self, line);

          self.is_preload = tmp;

          if line.is_empty() {
            return ierror!("Missing param in line {}", pos);
          }

          params.insert(format!("{i}$ROBSON"), line);
          pos += 1;
        }

        compiler.set_offset(command_number);
        compiler.set_preload(true);
        compiler.inner_in(self.inner);
        compiler.set_files(self.files.clone());
        compiler.set_macro_params(params);

        compiler
          .compiled_stack(self.compiled_stack.clone(), &self.path)?;

        match compiler.compile() {
          Ok(_) => {
            // inherit the compiled files
            self.files = compiler.files.clone();

            if compiler.is_static {
              self.files.insert(
                path.to_owned(),
                (compiler.current_command, params_count.len()),
              );
            }
            Ok((compiler.current_command, params_count.len()))
          }
          Err(err) => Err(err),
        }
      }
    }
  }

  pub fn get_cached_robsons_size(
    &mut self,
    path: &str,
    command_number: usize,
  ) -> Result<(usize, usize), IError> {
    match self.files.get(path) {
      Some(a) => Ok(*a),
      None => {
        // compile file and cache it
        let mut compiler = compiler!(path, self.infra);

        self.infra.color_print(format!("Preloading {path}\n"), 14);

        compiler.set_offset(command_number);
        compiler.set_preload(true);
        compiler.inner_in(self.inner);
        compiler.set_files(self.files.clone());

        compiler
          .compiled_stack(self.compiled_stack.clone(), &self.path)?;

        match compiler.compile() {
          Ok(_) => {
            // inherit the compiled files
            self.files = compiler.files.clone();
            if compiler.is_static {
              self.files.insert(
                path.to_owned(),
                (compiler.current_command, 0),
              );
            }

            Ok((compiler.current_command, 0))
          }
          Err(err) => Err(err),
        }
      }
    }
  }

  pub fn start_command_alias(&mut self) -> Result<(), IError> {
    struct LastCommand {
      value: u8,
      pos: usize,
    }
    let mut command_number = 0;
    let mut last_command = LastCommand { pos: 0, value: 0 };
    if self.macro_params.is_some() {
      self.pos += 1;
    }

    loop {
      if self.verify_index_overflow(self.pos) {
        break;
      }

      let i = &self.lines[self.pos];

      let mut string = Self::remove_comments(i).trim().to_owned();

      if string.is_empty() {
        self.pos += 1;
        continue;
      }

      replace_params!(self, string);

      if string == "SEMPRE#ROBSON" {
        if self.macro_jump.sx != 0 {
          self.pos = self.macro_jump.top().into();
          continue;
        } else {
          self.pos += 1;
          continue;
        }
      }
      if string == "PARE#ROBSON" {
        self.macro_current.pop();
        self.pos += 1;
        continue;
      }

      //add alias if it is an alias
      if string.ends_with(':') {
        let value = string.trim().replace(':', "");
        if self.names.get(&value).is_some() {
          return ierror!("Duplicated alias: {}", value);
        }
        if self.debug {
          self.infra.println(format!("{}: {}", value, self.pos + 1));
        }
        self.names.insert(value, command_number + self.offset);
      } else {
        //if is not an check what it is
        if string.starts_with("robsons") {
          //if is an include compile the include to get the correct value of the aliases
          let splited: Vec<&str> = string.split(' ').collect();
          if splited.len() != 2 {
            return ierror!("Malformated robsons");
          }
          let path = splited[1];

          // get offset from cache if possible
          let (new_offset, _) = match self
            .get_cached_robsons_size(path, command_number)
          {
            Ok(a) => a,
            Err(err) => return Err(err),
          };

          command_number += new_offset;
        } else if string.contains("robsons") {
          let split: Vec<&str> = string.split('[').collect();
          let split2: Vec<&str> = string.split(']').collect();
          if split2.len() == 2 && split.len() == 2 {
            let inside: Vec<&str> =
              split2[0][1..].split("robsons").collect();

            if inside.len() != 2 {
              return ierror!(
                "Malformated robsons macro at {}",
                string
              );
            }
            if inside[1].trim().is_empty() {
              return ierror!(
                "Malformated robsons macro at {}",
                string
              );
            }

            let (new_offset, params_count) = match self
              .get_cached_macro_size(
                inside[1].trim(),
                command_number,
                self.pos,
              ) {
              Ok(a) => a,
              Err(err) => return Err(err),
            };
            last_command.value = 0;
            last_command.pos = self.pos;
            self.pos += params_count;
            command_number += new_offset
          }
        } else if string.starts_with("robson") {
          // if is a command just add it
          command_number += 1;
          let mut opcode: u8 = 0;
          let spaces: Vec<&str> = string.split(' ').collect();

          for i in spaces {
            if i != "robson" {
              return ierror!(
                "invalid token for opcode in line {}, '{}'",
                self.pos + 1,
                i
              );
            }
            opcode += 1;
          }
          last_command.value = opcode;
          last_command.pos = self.pos;
        } else if last_command.value == 3
          && last_command.pos + 1 != self.pos
        {
          command_number += 1;
        }
      }
      self.pos += 1;
    }
    self.macro_current = Stack::new();
    self.macro_jump = Stack::new();
    self.pos = 0;
    Ok(())
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
      return ierror!(
        "Malformated param at line {}\n'{}' - {}",
        self.pos,
        self.lines[self.pos],
        self.path
      );
    }

    if splited.len() > 3 {
      return ierror!(
        "Malformated comeu in line {}\n'{}' - {}",
        self.pos,
        parameter,
        self.path
      );
    }

    let mut convert = false;
    if splited.len() == 3 {
      if splited[2] == "robson" {
        convert = true;
      } else {
        return ierror!(
          "Malformated param at line {}, expected 'robson'\n{}",
          self.pos,
          self.lines[self.pos]
        );
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
          return ierror!(
            "Malformated name in command at {}, '{}'",
            self.pos,
            value
          );
        }
        let value = value.replace(':', "");

        let a = self.names.get(&value).ok_or_else(|| {
          IError::message(&format!(
            "Cant find '{}' in {}",
            value, self.path
          ))
        })?;
        Ok(((*a as u32).into(), 0, 0, convert))
      }
      "penetrou" => {
        let value = splited[1].trim().parse::<u32>()?;
        Ok((value.into(), 3, 0, convert))
      }
      token => ierror!(
        "Unexpect token for param at line {}, '{}'",
        self.pos,
        token
      ),
    }
  }
}
