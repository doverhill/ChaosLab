use std::cell::RefMut;

use protocol_console::*;
use sdl2::pixels::Color as SdlColor;
use sdl2::rect::Point as SdlPoint;
use sdl2::keyboard::Keycode as SdlKeycode;
use sdl2::rect::Rect;
use sdl2::sys::posix_memalign;
use crate::application::Client;

pub fn convert_color_Console2SDL(color: Color) -> SdlColor {
    SdlColor { a: color.alpha, r: color.red, g: color.green, b: color.blue }
}

pub fn convert_point_Console2SDL(point: Point) -> SdlPoint {
    SdlPoint::new(point.x as i32, point.y as i32)
}

pub fn convert_key_code_SDL2Console(keycode: SdlKeycode) -> KeyCode {
    match keycode {
        SdlKeycode::A => KeyCode::A,
        _ => KeyCode::B
    }
}

pub fn draw_text(client: RefMut<Client>, text: &String) {

}

pub fn draw_pixel(mut client: RefMut<Client>, color: Color, position: Point) {
    client.surface.fill_rect(Rect::new(position.x as i32, position.y as i32, 1, 1), convert_color_Console2SDL(color));
}