use std::time::Duration;
use crossterm::event::{read, Event, KeyCode, KeyEvent, KeyModifiers, poll};
use crossterm::terminal;

pub struct Keypad{}

impl Keypad {

    pub fn pressed() -> Option<u32> {
        terminal::enable_raw_mode().expect("Could not turn on Raw mode");
        if let Ok(true) = poll(Duration::from_millis(100)) {
            let char = match read().unwrap() {
                Event::Key(KeyEvent {
                    code: KeyCode::Char(c),
                    modifiers: KeyModifiers::NONE,
                ..}) => Some(c),
                _ => None
            };
    
            match char {
                Some('1') => Some(0x1),
                Some('2') => Some(0x2),
                Some('3') => Some(0x3),
                Some('4') => Some(0xc),
                Some('q') => Some(0x4),
                Some('w') => Some(0x5),
                Some('e') => Some(0x6),
                Some('r') => Some(0xd),
                Some('a') => Some(0x7),
                Some('s') => Some(0x8),
                Some('d') => Some(0x9),
                Some('f') => Some(0xe),
                Some('z') => Some(0xa),
                Some('x') => Some(0x0),
                Some('c') => Some(0xb),
                Some('v') => Some(0xf),
                _ => None
            }
        }
        else {
            None
        }

    }

    fn map_key(c: char) -> Option<u32> {
        match c {
            '1'..='f' => c.to_digit(10),
            _ => None
        }
    }

}