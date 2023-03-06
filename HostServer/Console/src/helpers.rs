use protocol_console::*;
use sdl2::pixels::Color as SdlColor;
use sdl2::rect::Point as SdlPoint;

pub fn convert_color(color: Color) -> SdlColor {
    SdlColor { a: color.alpha, r: color.red, g: color.green, b: color.blue }
}

pub fn convert_point(point: Point) -> SdlPoint {
    SdlPoint::new(point.x as i32, point.y as i32)
}