#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Style {
    pub foreground: Option<Rgb>,
    pub bold: bool,
    pub dim: bool,
}

impl Style {
    pub const fn color(foreground: Rgb) -> Self {
        Self {
            foreground: Some(foreground),
            bold: false,
            dim: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Cell {
    pub glyph: char,
    pub style: Style,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            glyph: ' ',
            style: Style::default(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Canvas {
    width: u16,
    height: u16,
    cells: Vec<Cell>,
    depth: Vec<i32>,
}

impl Canvas {
    pub fn new(width: u16, height: u16) -> Self {
        let len = usize::from(width) * usize::from(height);
        Self {
            width,
            height,
            cells: vec![Cell::default(); len],
            depth: vec![i32::MIN; len],
        }
    }

    pub fn width(&self) -> u16 {
        self.width
    }

    pub fn height(&self) -> u16 {
        self.height
    }

    pub fn cells(&self) -> &[Cell] {
        &self.cells
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        if self.width == width && self.height == height {
            return;
        }
        *self = Self::new(width, height);
    }

    pub fn clear(&mut self) {
        self.cells.fill(Cell::default());
        self.depth.fill(i32::MIN);
    }

    pub fn get(&self, x: u16, y: u16) -> Option<Cell> {
        self.index(i32::from(x), i32::from(y))
            .map(|index| self.cells[index])
    }

    pub fn put(&mut self, x: i32, y: i32, glyph: char, style: Style) {
        if let Some(index) = self.index(x, y) {
            self.cells[index] = Cell { glyph, style };
        }
    }

    pub fn plot(&mut self, x: i32, y: i32, depth: f64, glyph: char, style: Style) {
        if !depth.is_finite() {
            return;
        }
        let quantized_depth = (depth * 1_000_000.0) as i32;
        if let Some(index) = self.index(x, y)
            && quantized_depth >= self.depth[index]
        {
            self.depth[index] = quantized_depth;
            self.cells[index] = Cell { glyph, style };
        }
    }

    pub fn text(&mut self, x: i32, y: i32, text: &str, style: Style) {
        for (offset, glyph) in text.chars().enumerate() {
            self.put(x + offset as i32, y, glyph, style);
        }
    }

    pub fn fill_rect(&mut self, x: i32, y: i32, width: u16, height: u16, style: Style) {
        for row in 0..height {
            for column in 0..width {
                self.put(x + i32::from(column), y + i32::from(row), ' ', style);
            }
        }
    }

    pub fn visible_cells(&self) -> usize {
        self.cells.iter().filter(|cell| cell.glyph != ' ').count()
    }

    fn index(&self, x: i32, y: i32) -> Option<usize> {
        if x < 0 || y < 0 || x >= i32::from(self.width) || y >= i32::from(self.height) {
            return None;
        }
        Some(y as usize * usize::from(self.width) + x as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clips_writes_and_prefers_nearer_depth() {
        let mut canvas = Canvas::new(3, 2);
        canvas.put(-1, 0, 'X', Style::default());
        canvas.plot(1, 1, 0.2, '.', Style::default());
        canvas.plot(1, 1, 0.1, '#', Style::default());
        canvas.plot(1, 1, 0.3, '@', Style::default());

        assert_eq!(canvas.visible_cells(), 1);
        assert_eq!(canvas.get(1, 1).unwrap().glyph, '@');
    }

    #[test]
    fn tiny_canvases_are_valid() {
        let mut canvas = Canvas::new(0, 0);
        canvas.put(0, 0, '#', Style::default());
        canvas.resize(1, 1);
        canvas.put(0, 0, '#', Style::default());
        assert_eq!(canvas.visible_cells(), 1);
    }
}
