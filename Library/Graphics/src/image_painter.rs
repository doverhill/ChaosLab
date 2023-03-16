use protocol_console::*;

pub struct ImagePainter {
    size: Size,
    pixels: Vec<Color>,
}

impl ImagePainter {
    pub fn new(width: usize, height: usize, color: Color) -> Self {
        Self {
            size: Size { width: width as u64, height: height as u64 },
            pixels: vec![color; width * height],
        }
    }

    pub fn to_image(self) -> Image {
        Image {
            size: self.size,
            pixels: self.pixels
        }
    }

    pub fn draw_filled_box_sized(&mut self, position: Point, size: Size, color: Color) {
        self.draw_filled_box_xy(position, Point { x: position.x + size.width as i64, y: position.y + size.height as i64}, color);
    }

    pub fn draw_filled_box_xy(&mut self, top_left: Point, bottom_right: Point, color: Color) {
        assert!(bottom_right.y >= top_left.y);
        assert!(bottom_right.x >= top_left.x);

        for y in top_left.y..=bottom_right.y {
            for x in top_left.x..=bottom_right.x {
                self.pixels[y as usize * self.size.width as usize + x as usize] = color;
            }
        }
    }

    pub fn draw_frame_sized(&mut self, position: Point, size: Size, color: Color) {
        self.draw_frame_xy(position, Point { x: position.x + size.width as i64, y: position.y + size.height as i64}, color);
    }

    pub fn draw_frame_xy(&mut self, top_left: Point, bottom_right: Point, color: Color) {
        assert!(bottom_right.y >= top_left.y);
        assert!(bottom_right.x >= top_left.x);

        for y in top_left.y..=bottom_right.y {
            self.pixels[y as usize * self.size.width as usize + top_left.x as usize] = color;
            self.pixels[y as usize * self.size.width as usize + bottom_right.x as usize] = color;
        }

        for x in top_left.x..=bottom_right.x {
            self.pixels[top_left.y as usize * self.size.width as usize + x as usize] = color;
            self.pixels[bottom_right.y as usize * self.size.width as usize + x as usize] = color;
        }
    }
}
