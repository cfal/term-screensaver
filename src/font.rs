use crate::math::Vec3;

pub const GLYPH_WIDTH: usize = 5;
pub const GLYPH_HEIGHT: usize = 7;

pub fn bitmap(glyph: char) -> Option<[u8; GLYPH_HEIGHT]> {
    Some(match glyph.to_ascii_uppercase() {
        'A' => [14, 17, 17, 31, 17, 17, 17],
        'B' => [30, 17, 17, 30, 17, 17, 30],
        'C' => [15, 16, 16, 16, 16, 16, 15],
        'D' => [30, 17, 17, 17, 17, 17, 30],
        'E' => [31, 16, 16, 30, 16, 16, 31],
        'F' => [31, 16, 16, 30, 16, 16, 16],
        'G' => [15, 16, 16, 23, 17, 17, 14],
        'H' => [17, 17, 17, 31, 17, 17, 17],
        'I' => [31, 4, 4, 4, 4, 4, 31],
        'J' => [7, 2, 2, 2, 18, 18, 12],
        'K' => [17, 18, 20, 24, 20, 18, 17],
        'L' => [16, 16, 16, 16, 16, 16, 31],
        'M' => [17, 27, 21, 21, 17, 17, 17],
        'N' => [17, 25, 21, 19, 17, 17, 17],
        'O' => [14, 17, 17, 17, 17, 17, 14],
        'P' => [30, 17, 17, 30, 16, 16, 16],
        'Q' => [14, 17, 17, 17, 21, 18, 13],
        'R' => [30, 17, 17, 30, 20, 18, 17],
        'S' => [15, 16, 16, 14, 1, 1, 30],
        'T' => [31, 4, 4, 4, 4, 4, 4],
        'U' => [17, 17, 17, 17, 17, 17, 14],
        'V' => [17, 17, 17, 17, 17, 10, 4],
        'W' => [17, 17, 17, 21, 21, 27, 17],
        'X' => [17, 17, 10, 4, 10, 17, 17],
        'Y' => [17, 17, 10, 4, 4, 4, 4],
        'Z' => [31, 1, 2, 4, 8, 16, 31],
        '0' => [14, 17, 19, 21, 25, 17, 14],
        '1' => [4, 12, 4, 4, 4, 4, 14],
        '2' => [14, 17, 1, 2, 4, 8, 31],
        '3' => [30, 1, 1, 14, 1, 1, 30],
        '4' => [2, 6, 10, 18, 31, 2, 2],
        '5' => [31, 16, 16, 30, 1, 1, 30],
        '6' => [14, 16, 16, 30, 17, 17, 14],
        '7' => [31, 1, 2, 4, 8, 8, 8],
        '8' => [14, 17, 17, 14, 17, 17, 14],
        '9' => [14, 17, 17, 15, 1, 1, 14],
        ':' => [0, 4, 4, 0, 4, 4, 0],
        ' ' => [0; GLYPH_HEIGHT],
        _ => return None,
    })
}

pub fn points(text: &str) -> Vec<Vec3> {
    let glyph_count = text.chars().count();
    if glyph_count == 0 {
        return Vec::new();
    }
    let total_width = glyph_count * (GLYPH_WIDTH + 1) - 1;
    let mut points = Vec::new();
    for (glyph_index, glyph) in text.chars().enumerate() {
        let Some(rows) = bitmap(glyph) else {
            continue;
        };
        for (row, mask) in rows.into_iter().enumerate() {
            for column in 0..GLYPH_WIDTH {
                if mask & (1 << (GLYPH_WIDTH - column - 1)) == 0 {
                    continue;
                }
                let x = (glyph_index * (GLYPH_WIDTH + 1) + column) as f64
                    - (total_width - 1) as f64 * 0.5;
                let y = (GLYPH_HEIGHT - 1) as f64 * 0.5 - row as f64;
                points.push(Vec3::new(x, y, 0.0));
            }
        }
    }
    if let (Some(min), Some(max)) = (
        points.iter().map(|point| point.x).reduce(f64::min),
        points.iter().map(|point| point.x).reduce(f64::max),
    ) {
        let center = (min + max) * 0.5;
        for point in &mut points {
            point.x -= center;
        }
    }
    points
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn covers_alphanumerics_and_clock_punctuation() {
        for glyph in "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789:".chars() {
            assert!(bitmap(glyph).is_some(), "missing {glyph}");
            assert!(!points(&glyph.to_string()).is_empty(), "blank {glyph}");
        }
    }

    #[test]
    fn centers_text_geometry() {
        let points = points("12:34");
        let min = points
            .iter()
            .map(|point| point.x)
            .fold(f64::INFINITY, f64::min);
        let max = points
            .iter()
            .map(|point| point.x)
            .fold(f64::NEG_INFINITY, f64::max);
        assert!((min + max).abs() < 1.0);
    }
}
