
pub mod keypad {
    use crossterm::event::{read, Event, KeyCode, KeyEvent, KeyModifiers};
    pub struct Keypad{}

    impl Keypad {
       pub fn new() -> Keypad {
            Keypad {}
        }

        pub fn pressed() -> Option<u32> {
            match read().unwrap() {
                Event::Key(KeyEvent {
                    code: KeyCode::Char(c),
                    modifiers: KeyModifiers::NONE,
                ..}) => Self::map_key(c),
                _ => None
            }
        }

        fn map_key(c: char) -> Option<u32> {
            match c {
                '1'..='f' => c.to_digit(10),
                _ => None
            }
        }

    }
}