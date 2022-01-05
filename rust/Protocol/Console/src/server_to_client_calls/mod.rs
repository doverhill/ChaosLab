pub mod key_pressed;
pub use key_pressed::CONSOLE_KEY_PRESSED_SERVER_TO_CLIENT_MESSAGE;

pub mod key_released;
pub use key_released::CONSOLE_KEY_RELEASED_SERVER_TO_CLIENT_MESSAGE;

pub mod text_available;
pub use text_available::CONSOLE_TEXT_AVAILABLE_SERVER_TO_CLIENT_MESSAGE;

pub mod pointer_moved;
pub use pointer_moved::CONSOLE_POINTER_MOVED_SERVER_TO_CLIENT_MESSAGE;

pub mod pointer_button_pressed;
pub use pointer_button_pressed::CONSOLE_POINTER_BUTTON_PRESSED_SERVER_TO_CLIENT_MESSAGE;

pub mod pointer_button_released;
pub use pointer_button_released::CONSOLE_POINTER_BUTTON_RELEASED_SERVER_TO_CLIENT_MESSAGE;

