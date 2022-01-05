pub mod get_capabilities;
pub use get_capabilities::CONSOLE_GET_CAPABILITIES_CLIENT_TO_SERVER_MESSAGE;

pub mod set_text_color;
pub use set_text_color::CONSOLE_SET_TEXT_COLOR_CLIENT_TO_SERVER_MESSAGE;

pub mod set_text_cursor_position;
pub use set_text_cursor_position::CONSOLE_SET_TEXT_CURSOR_POSITION_CLIENT_TO_SERVER_MESSAGE;

pub mod write_text;
pub use write_text::CONSOLE_WRITE_TEXT_CLIENT_TO_SERVER_MESSAGE;

pub mod render_bitmap_patches;
pub use render_bitmap_patches::CONSOLE_RENDER_BITMAP_PATCHES_CLIENT_TO_SERVER_MESSAGE;

