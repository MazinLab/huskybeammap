#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, facet::Facet)]
pub enum Draw {
    Milo,
    Dvd,
    Rectangle { width: usize, height: usize },
}
#[derive(Clone, Debug, PartialEq, Eq, facet::Facet)]
pub struct Movement {
    pub position: isize,
    pub pixels: isize,
    pub frames: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, facet::Facet)]
pub struct Object {
    pub start: Option<u64>,
    pub lifetime: u64,
    pub draw: Draw,
    pub x: Movement,
    pub y: Movement,
}

#[derive(Clone, Debug, facet::Facet)]
pub struct StatusMessage {
    pub width: usize,
    pub height: usize,
    pub frame: u64,
    pub objects: usize,
    pub frame_rate: u32,
    pub frame_time: f32,
}

impl Object {
    pub fn position(&self, frame: u64) -> (isize, isize) {
        let start = self.start.unwrap_or(frame);
        let start = if start < frame { start } else { frame };
        let dx = ((frame - start) / self.x.frames) as isize * self.x.pixels;
        let dy = ((frame - start) / self.y.frames) as isize * self.y.pixels;
        (self.x.position + dx, self.y.position + dy)
    }

    pub fn repack(self, frame: u64) -> Option<Self> {
        let repacked = Object {
            start: Some(self.start.unwrap_or(frame)),
            lifetime: self.lifetime,
            draw: self.draw,
            x: self.x,
            y: self.y,
        };

        if frame >= repacked.start.unwrap() + repacked.lifetime {
            None
        } else {
            Some(repacked)
        }
    }
}
