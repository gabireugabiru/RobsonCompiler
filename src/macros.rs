macro_rules! compiler {
  ($a:expr, $b:expr) => {
    match crate::compiler::Compiler::new($a.into(), $b.clone_self()) {
      Ok(a) => a,
      Err(err) => {
        if err.to_string().contains("os error 2") {
          return Err(crate::data_struct::IError::message(format!(
            "No such file '{}' (os error 2)",
            $a
          )));
        } else {
          return Err(err);
        }
      }
    }
  };
}

macro_rules! ierror {
    ($arg:literal) => {
        Err(crate::data_struct::IError::message($arg))
      };
    ($($arg:tt)*) => {
        Err(crate::data_struct::IError::message(
        format!($($arg)*)
        ))
    };
}

macro_rules! replace_params {
  ($self:expr, $string:ident) => {
    if $string.contains("$ROBSON") || $string.contains("?ROBSON") {
      if let Some(macro_params) = &mut $self.macro_params {
        let mut should_pop = false;
        let current = match $self.macro_current.top() {
          Ok(a) => {
            should_pop = true;
            a
          }
          Err(_) => 0,
        };
        let (str, has_next, is_expr) =
          crate::utils::convert_macro_robson(
            $string.to_string(),
            &macro_params,
            current,
          )?;

        if is_expr {
          if should_pop {
            $self.macro_current.pop();
          }

          if has_next {
            $self.macro_current.push(current + 1);
          }

          let b = $string.split(" ").collect::<Vec<&str>>();

          if b.len() != 2 {
            return ierror!(
              "Invalid macro expression at {}",
              $self.pos
            );
          }

          macro_params.insert(b[1].replace("$", "?"), str);
          match $self.macro_jump.top() {
            Ok(x) => {
              if x != $self.pos {
                $self.macro_jump.push($self.pos);
              } else if !has_next {
                $self.macro_jump.pop();
              }
            }
            Err(_) => {
              $self.macro_jump.push($self.pos);
            }
          }

          $self.pos += 1;
          continue;
        } else {
          $string = str;
        }
      }
    }
  };
}

macro_rules! sanitize_param {
  ($self:ident,$string:ident) => {
    if $string.contains("lambeu") {
      let result = $self.get_kind_value(&$string);
      if let Ok((true_value, _, _, _)) = &result {
        if true_value.r#type != Type::Usigned {
          return ierror!(
            "Invalid lambeu at command of line {}",
            $self.pos - 1
          );
        }
        $string =
          format!("comeu {}", u32::from_be_bytes(true_value.value))
      }
      if let Err(err) = result {
        if err.error.contains("Cant find") && $self.is_preload {
          $string = format!("comeu 1");
        }
      }
    }
  };
}

pub(crate) use compiler;
pub(crate) use ierror;
pub(crate) use replace_params;
pub(crate) use sanitize_param;
