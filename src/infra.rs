use std::{
  io::{stdin, stdout, StdoutLock},
  time::Duration,
};

use crossterm::{
  cursor,
  event::{poll, read, Event, KeyCode},
  queue,
  style::{
    Color, ResetColor, SetBackgroundColor, SetForegroundColor,
  },
  terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
};

use std::io::Write;

use crate::{data_struct::IError, Infra};

pub struct RunInfra<'a> {
  stdout: StdoutLock<'a>,
}
impl<'a> RunInfra<'a> {
  pub fn new() -> Box<Self> {
    Box::new(Self {
      stdout: stdout().lock(),
    })
  }
}
impl<'a> Infra for RunInfra<'a> {
  #[inline(always)]
  fn print(&mut self, to_print: &[u8]) {
    self.stdout.write(&to_print).unwrap();
  }
  fn println(&mut self, to_print: String) {
    println!("{to_print}");
  }
  fn read_line(&mut self) -> Result<String, std::io::Error> {
    let mut buffer = String::new();
    stdin().read_line(&mut buffer)?;
    Ok(buffer)
  }
  fn flush(&mut self) {
    self.stdout.flush().unwrap();
  }
  fn clear_all(&mut self) -> Result<(), IError> {
    queue!(self.stdout, Clear(ClearType::All))?;
    Ok(())
  }
  fn clear_purge(&mut self) -> Result<(), IError> {
    queue!(self.stdout, Clear(ClearType::Purge))?;
    Ok(())
  }
  fn enable_raw_mode(&self) -> Result<(), IError> {
    enable_raw_mode()?;
    Ok(())
  }
  fn disable_raw_mode(&self) -> Result<(), IError> {
    disable_raw_mode()?;
    Ok(())
  }
  fn poll(&self, duration: u64) -> Result<u32, IError> {
    let mut code = 0;
    if poll(Duration::from_millis(duration))? {
      match read()? {
        Event::Key(key) => {
          code = match &key.code {
            KeyCode::Char(char) => {
              let mut bytes = [0, 0, 0, 0];
              char.encode_utf8(&mut bytes);

              let mut zeroes = 0;

              for i in bytes {
                if i == 0 {
                  zeroes += 1;
                }
              }

              let a = u32::from_be_bytes(bytes) >> 8 * zeroes;
              a
            }
            KeyCode::Esc => 10000,
            KeyCode::BackTab => 10001,
            KeyCode::Backspace => 10002,
            KeyCode::Delete => 10003,
            KeyCode::Down => 10004,
            KeyCode::End => 10005,
            KeyCode::Enter => 10006,
            KeyCode::Insert => 10007,
            KeyCode::Left => 10008,
            KeyCode::PageDown => 10009,
            KeyCode::PageUp => 10010,
            KeyCode::Right => 10011,
            KeyCode::Tab => 10012,
            KeyCode::Up => 10013,
            _ => 0,
          }
        }
        _ => {}
      }
    }
    Ok(code)
  }
  fn hide_cursor(&mut self) -> Result<(), IError> {
    queue!(self.stdout, cursor::Hide)?;
    Ok(())
  }
  fn show_cursor(&mut self) -> Result<(), IError> {
    queue!(self.stdout, cursor::Show)?;
    Ok(())
  }
  fn move_cursor(&mut self, x: u32, y: u32) -> Result<(), IError> {
    queue!(self.stdout, cursor::MoveTo(x as u16, y as u16))?;
    Ok(())
  }
  fn use_color(&mut self, color: u32) -> Result<(), IError> {
    if color < 256 {
      queue!(
        self.stdout,
        SetForegroundColor(Color::AnsiValue(color as u8))
      )?;
    } else {
      queue!(self.stdout, ResetColor)?;
    }

    Ok(())
  }
  fn use_background(&mut self, color: u32) -> Result<(), IError> {
    if color < 256 {
      queue!(
        self.stdout,
        SetBackgroundColor(Color::AnsiValue(color as u8))
      )?;
    } else {
      queue!(self.stdout, ResetColor)?;
    }
    Ok(())
  }
}
