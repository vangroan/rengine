//! Mapping user input to game actions.
use glutin::{MouseButton, VirtualKeyCode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub struct InputMap<T> {
    pub(crate) mapping: HashMap<VirtualKeyCode, T>,
}

impl<T> InputMap<T> {
    pub fn new() -> Self {
        Default::default()
    }
}

impl<T> Default for InputMap<T> {
    fn default() -> Self {
        InputMap {
            mapping: HashMap::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
pub enum UserInput {
    Keyboard(VirtualKeyCode),
    Mouse(MouseButton),
}

#[cfg(test)]
mod test {
    use super::*;
    use serde::{Deserialize, Serialize};
    use std::fs;
    use std::io::prelude::*;
    use toml;

    #[derive(Serialize, Deserialize)]
    enum TestAction {
        WalkForward,
        Jump,
    }

    #[test]
    fn test_serialize() {
        if let Err(err) = fs::create_dir(".tmp") {
            println!("{}", err);
        }

        let mut input_map: InputMap<TestAction> = InputMap::new();
        input_map
            .mapping
            .insert(VirtualKeyCode::A, TestAction::WalkForward);
        // input_map.mapping.insert(
        //     UserInput::Keyboard(VirtualKeyCode::W),
        //     TestAction::WalkForward,
        // );
        // input_map
        //     .mapping
        //     .insert(UserInput::Mouse(MouseButton::Left), TestAction::Jump);

        let data = vec![UserInput::Mouse(MouseButton::Left)];
        let input_map_data = toml::to_string(&data).unwrap();
        let mut file = fs::File::create(".tmp/input-map.toml").unwrap();
        file.write_all(&input_map_data.into_bytes()).unwrap();
    }
}
