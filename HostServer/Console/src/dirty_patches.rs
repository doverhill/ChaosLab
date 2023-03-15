use sdl2::rect::Rect;

pub struct DirtyPatches {
    pub patches: Vec<Rect>,
}

impl DirtyPatches {
    pub fn new() -> Self {
        Self { patches: Vec::new() }
    }

    pub fn add_dirty(&mut self, patch: Rect) {
        // FIXME figure out intersections and split into several smaller patches
        self.patches.push(patch);
    }
}
