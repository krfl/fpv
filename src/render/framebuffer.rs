/// RGB color.
#[derive(Clone, Copy, Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub const BLACK: Self = Self::new(0, 0, 0);

    pub fn scale(self, factor: f32) -> Self {
        Self {
            r: (self.r as f32 * factor).min(255.0) as u8,
            g: (self.g as f32 * factor).min(255.0) as u8,
            b: (self.b as f32 * factor).min(255.0) as u8,
        }
    }

    #[allow(dead_code)]
    pub fn luminance(self) -> f32 {
        (0.299 * self.r as f32 + 0.587 * self.g as f32 + 0.114 * self.b as f32) / 255.0
    }

}

/// Software framebuffer with color and depth.
pub struct Framebuffer {
    pub width: u32,
    pub height: u32,
    pub color: Vec<Color>,
    pub depth: Vec<f32>,
}

impl Framebuffer {
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width * height) as usize;
        Self {
            width,
            height,
            color: vec![Color::BLACK; size],
            depth: vec![f32::INFINITY; size],
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if self.width == width && self.height == height {
            return;
        }
        self.width = width;
        self.height = height;
        let size = (width * height) as usize;
        self.color.resize(size, Color::BLACK);
        self.depth.resize(size, f32::INFINITY);
    }

    pub fn clear(&mut self, sky_color: Color) {
        self.color.fill(sky_color);
        self.depth.fill(f32::INFINITY);
    }

    #[inline]
    #[allow(dead_code)]
    pub fn set_pixel(&mut self, x: u32, y: u32, z: f32, color: Color) {
        if x >= self.width || y >= self.height {
            return;
        }
        let idx = (y * self.width + x) as usize;
        if z < self.depth[idx] {
            self.depth[idx] = z;
            self.color[idx] = color;
        }
    }

    #[inline]
    #[allow(dead_code)]
    pub fn get_pixel(&self, x: u32, y: u32) -> Color {
        self.color[(y * self.width + x) as usize]
    }

    /// Merge another framebuffer into this one using depth comparison.
    #[allow(dead_code)]
    pub fn merge(&mut self, other: &Framebuffer) {
        for i in 0..self.depth.len() {
            if other.depth[i] < self.depth[i] {
                self.depth[i] = other.depth[i];
                self.color[i] = other.color[i];
            }
        }
    }
}
