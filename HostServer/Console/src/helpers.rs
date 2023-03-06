use std::cell::RefMut;

use protocol_console::*;
use sdl2::pixels::Color as SdlColor;
use sdl2::keyboard::Keycode as SdlKeycode;
use sdl2::rect::Rect;
use crate::application::Client;

pub fn convert_color_console_to_sdl(color: Color) -> SdlColor {
    SdlColor { a: color.alpha, r: color.red, g: color.green, b: color.blue }
}

// pub fn convert_point_console_to_sdl(point: Point) -> SdlPoint {
//     SdlPoint::new(point.x as i32, point.y as i32)
// }

pub fn convert_key_code_sdl_to_console(keycode: SdlKeycode) -> KeyCode {
    match keycode {
        SdlKeycode::A => KeyCode::A,
        _ => KeyCode::B
    }
}

pub fn draw_text(_client: RefMut<Client>, _text: &String) {

}

pub fn draw_pixel(mut client: RefMut<Client>, color: Color, position: Point) {
    client.surface.fill_rect(Rect::new(position.x as i32, position.y as i32, 1, 1), convert_color_console_to_sdl(color)).unwrap();
}