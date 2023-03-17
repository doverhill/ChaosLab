use std::cell::RefMut;

use crate::application::Client;
use protocol_console::*;
use protocol_data::*;
use sdl2::keyboard::Keycode as SdlKeycode;
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color as SdlColor;
use sdl2::rect::Rect;
use sdl2::surface::Surface;
use sdl2::ttf::Font;

pub fn convert_color_console_to_sdl(color: Color) -> SdlColor {
    SdlColor {
        a: color.alpha,
        r: color.red,
        g: color.green,
        b: color.blue,
    }
}

// pub fn convert_point_console_to_sdl(point: Point) -> SdlPoint {
//     SdlPoint::new(point.x as i32, point.y as i32)
// }

pub fn convert_key_code_sdl_to_console(keycode: SdlKeycode) -> KeyCode {
    match keycode {
        SdlKeycode::A => KeyCode::A,
        SdlKeycode::B => KeyCode::B,
        SdlKeycode::C => KeyCode::C,
        SdlKeycode::D => KeyCode::D,
        SdlKeycode::Return => KeyCode::Enter,
        SdlKeycode::Backspace => KeyCode::Backspace,
        SdlKeycode::Delete => KeyCode::Delete,
        SdlKeycode::Home => KeyCode::Home,
        SdlKeycode::End => KeyCode::End,
        SdlKeycode::Up => KeyCode::UpArrow,
        SdlKeycode::Down => KeyCode::DownArrow,
        SdlKeycode::Left => KeyCode::LeftArrow,
        SdlKeycode::Right => KeyCode::RightArrow,
        _ => KeyCode::B,
    }
}

pub fn convert_mount_button_sdl_to_console(mouse_button: MouseButton) -> PointerButton {
    match mouse_button {
        MouseButton::Left => PointerButton::Left,
        MouseButton::Middle => PointerButton::Middle,
        MouseButton::Right => PointerButton::Right,
        _ => PointerButton::Left
    }
}

pub fn convert_image_to_surface<'a>(image: &'a Image) -> Surface<'a> {
    unsafe {
        let data_pointer = image.pixels.as_ptr() as *mut u8;
        let slice = core::slice::from_raw_parts_mut(data_pointer, image.size.width as usize * image.size.height as usize);
        Surface::from_data(slice, image.size.width as u32, image.size.height as u32, image.size.width as u32 * 4, sdl2::pixels::PixelFormatEnum::ARGB32).unwrap()
    }
}

pub fn draw_text(mut client: RefMut<Client>, glyph_size: Size, text_size: TextSize, font: &Font, text: &String) -> Rect {
    let mut min_column = i64::MAX;
    let mut min_row = i64::MAX;
    let mut max_column = 0i64;
    let mut max_row = 0i64;

    for character in text.chars() {
        if character == '\n' {
            client.text_position.column = 0;
            client.text_position.row += 1;
        } else {
            let x = client.text_position.column as u64 * glyph_size.width;
            let y = client.text_position.row as u64 * glyph_size.height;

            if client.text_position.column < min_column {
                min_column = client.text_position.column;
            }
            if client.text_position.column > max_column {
                max_column = client.text_position.column;
            }
            if client.text_position.row < min_row {
                min_row = client.text_position.row;
            }
            if client.text_position.row > max_row {
                max_row = client.text_position.row;
            }

            let rendering = font.render_char(character);
            let foreground_color = convert_color_console_to_sdl(Color {
                alpha: 255,
                red: 255,
                green: 255,
                blue: 255,
            });
            let background_color = convert_color_console_to_sdl(Color {
                alpha: 255,
                red: 0,
                green: 128,
                blue: 255,
            });
            let surface = rendering.shaded(foreground_color, background_color).unwrap();
            surface.blit(None, &mut client.surface, Rect::new(x as i32, y as i32, 0, 0)).unwrap();

            client.text_position.column += 1;
            if client.text_position.column == text_size.columns as i64 {
                client.text_position.column = 0;
                client.text_position.row += 1;
            }
        }

        if client.text_position.row == text_size.rows as i64 {
            // FIXME scroll here
            client.saved_text_position.row -= 1;
            client.text_position.row -= 1;
        }
    }

    Rect::new(min_column as i32 * glyph_size.width as i32, min_row as i32 * glyph_size.height as i32, (max_column - min_column + 1) as u32 * glyph_size.width as u32, (max_row - min_row + 1) as u32 * glyph_size.height as u32)
}

pub fn draw_pixel(mut client: RefMut<Client>, color: Color, position: Point) -> Rect {
    let pixel = Rect::new(position.x as i32, position.y as i32, 1, 1);
    client
        .surface
        .fill_rect(pixel, convert_color_console_to_sdl(color))
        .unwrap();
    pixel
}

pub fn draw_image(mut client: RefMut<Client>, image: &Image, position: Point) -> Rect {
    let surface = convert_image_to_surface(image);
    surface.blit(None, &mut client.surface, Rect::new(position.x as i32, position.y as i32, 0, 0)).unwrap();
    Rect::new(position.x as i32, position.y as i32, image.size.width as u32, image.size.height as u32)
}