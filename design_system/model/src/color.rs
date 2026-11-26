//! Colour maths: parsing, WCAG 2.2 contrast, APCA, OKLab and colour-blindness
//! simulation. All functions are pure and work on sRGB colours.

use std::fmt;

/// An sRGB colour with 8-bit channels and alpha.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Rgba {
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Parse `#rgb`, `#rrggbb` or `#rrggbbaa`.
    pub fn parse_hex(s: &str) -> Option<Self> {
        let hex = s.strip_prefix('#')?;
        if !hex.chars().all(|c| c.is_ascii_hexdigit()) {
            return None;
        }
        let byte = |i: usize| u8::from_str_radix(hex.get(i..i + 2)?, 16).ok();
        match hex.len() {
            3 => {
                let mut c = hex.chars().map(|c| c.to_digit(16).map(|d| (d * 17) as u8));
                Some(Self::new(c.next()??, c.next()??, c.next()??, 255))
            }
            6 => Some(Self::new(byte(0)?, byte(2)?, byte(4)?, 255)),
            8 => Some(Self::new(byte(0)?, byte(2)?, byte(4)?, byte(6)?)),
            _ => None,
        }
    }

    /// Channels as fractions of 1, gamma-encoded.
    fn unit(self) -> [f64; 3] {
        [self.r, self.g, self.b].map(|c| f64::from(c) / 255.0)
    }

    /// Channels in linear light (the WCAG 2.2 / sRGB transfer function).
    pub fn linear(self) -> [f64; 3] {
        self.unit().map(|c| {
            if c <= 0.04045 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        })
    }

    /// WCAG 2.2 relative luminance, 0 (black) to 1 (white).
    pub fn relative_luminance(self) -> f64 {
        let [r, g, b] = self.linear();
        0.2126 * r + 0.7152 * g + 0.0722 * b
    }

    /// Composite this colour over an opaque background.
    pub fn over(self, bg: Rgba) -> Rgba {
        let a = f64::from(self.a) / 255.0;
        let mix = |f: u8, b: u8| (f64::from(f) * a + f64::from(b) * (1.0 - a)).round() as u8;
        Rgba::new(mix(self.r, bg.r), mix(self.g, bg.g), mix(self.b, bg.b), 255)
    }
}

impl fmt::Display for Rgba {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{:02x}{:02x}{:02x}", self.r, self.g, self.b)?;
        if self.a != 255 {
            write!(f, "{:02x}", self.a)?;
        }
        Ok(())
    }
}

/// WCAG 2.2 contrast ratio between two colours, 1 to 21. Order doesn't matter.
pub fn contrast_ratio(a: Rgba, b: Rgba) -> f64 {
    let (la, lb) = (a.relative_luminance(), b.relative_luminance());
    let (hi, lo) = if la > lb { (la, lb) } else { (lb, la) };
    (hi + 0.05) / (lo + 0.05)
}

/// APCA lightness contrast (Lc) of `text` on `bg`, APCA-W3 0.0.98G-4g.
/// Positive for dark text on a light background, negative for the reverse.
/// Shown for information only: WCAG 3 is not final.
pub fn apca_lc(text: Rgba, bg: Rgba) -> f64 {
    fn y(c: Rgba) -> f64 {
        let [r, g, b] = c.unit().map(|v| v.powf(2.4));
        let y = 0.2126729 * r + 0.7151522 * g + 0.0721750 * b;
        if y < 0.022 {
            y + (0.022 - y).powf(1.414)
        } else {
            y
        }
    }
    let (yt, yb) = (y(text), y(bg));
    if (yb - yt).abs() < 0.0005 {
        return 0.0;
    }
    let out = if yb > yt {
        let s = (yb.powf(0.56) - yt.powf(0.57)) * 1.14;
        if s < 0.1 { 0.0 } else { s - 0.027 }
    } else {
        let s = (yb.powf(0.65) - yt.powf(0.62)) * 1.14;
        if s > -0.1 { 0.0 } else { s + 0.027 }
    };
    out * 100.0
}

/// A form of colour blindness to simulate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Vision {
    Normal,
    Protanopia,
    Deuteranopia,
    Tritanopia,
}

impl Vision {
    pub const ALL: [Vision; 4] = [
        Vision::Normal,
        Vision::Protanopia,
        Vision::Deuteranopia,
        Vision::Tritanopia,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Vision::Normal => "normal vision",
            Vision::Protanopia => "protanopia",
            Vision::Deuteranopia => "deuteranopia",
            Vision::Tritanopia => "tritanopia",
        }
    }

    /// Machado, Oliveira and Fernandes (2009) matrices at severity 1.0,
    /// applied in linear sRGB.
    fn matrix(self) -> [[f64; 3]; 3] {
        match self {
            Vision::Normal => [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            Vision::Protanopia => [
                [0.152286, 1.052583, -0.204868],
                [0.114503, 0.786281, 0.099216],
                [-0.003882, -0.048116, 1.051998],
            ],
            Vision::Deuteranopia => [
                [0.367322, 0.860646, -0.227968],
                [0.280085, 0.672501, 0.047413],
                [-0.011820, 0.042940, 0.968881],
            ],
            Vision::Tritanopia => [
                [1.255528, -0.076749, -0.178779],
                [-0.078411, 0.930809, 0.147602],
                [0.004733, 0.691367, 0.303900],
            ],
        }
    }
}

/// A colour in OKLab, as seen with the given vision.
pub fn oklab(c: Rgba, vision: Vision) -> [f64; 3] {
    let lin = c.linear();
    let m = vision.matrix();
    let [r, g, b] =
        [0, 1, 2].map(|i| (m[i][0] * lin[0] + m[i][1] * lin[1] + m[i][2] * lin[2]).clamp(0.0, 1.0));
    let l = (0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b).cbrt();
    let m = (0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b).cbrt();
    let s = (0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b).cbrt();
    [
        0.2104542553 * l + 0.7936177850 * m - 0.0040720468 * s,
        1.9779984951 * l - 2.4285922050 * m + 0.4505937099 * s,
        0.0259040371 * l + 0.7827717662 * m - 0.8086757660 * s,
    ]
}

/// Euclidean distance in OKLab between two colours seen with `vision`.
/// About 0.02 is a just-noticeable difference.
pub fn distance(a: Rgba, b: Rgba, vision: Vision) -> f64 {
    let (p, q) = (oklab(a, vision), oklab(b, vision));
    ((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2)).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex(s: &str) -> Rgba {
        Rgba::parse_hex(s).unwrap()
    }

    #[test]
    fn parses_and_prints_hex() {
        assert_eq!(hex("#fff"), Rgba::new(255, 255, 255, 255));
        assert_eq!(hex("#8a4b1f"), Rgba::new(0x8a, 0x4b, 0x1f, 255));
        assert_eq!(hex("#00000080").a, 0x80);
        assert_eq!(hex("#8A4B1F").to_string(), "#8a4b1f");
        assert_eq!(hex("#00000080").to_string(), "#00000080");
        for bad in ["8a4b1f", "#12", "#12345", "#gggggg", "#1234567"] {
            assert_eq!(Rgba::parse_hex(bad), None, "{bad}");
        }
    }

    // Values from the WCAG 2.2 definitions of relative luminance and
    // contrast ratio, and the WebAIM contrast checker.
    #[test]
    fn wcag_relative_luminance() {
        assert_eq!(hex("#000000").relative_luminance(), 0.0);
        assert!((hex("#ffffff").relative_luminance() - 1.0).abs() < 1e-9);
        assert!((hex("#808080").relative_luminance() - 0.2158605).abs() < 1e-6);
    }

    #[test]
    fn wcag_contrast_ratio() {
        let r = |a, b| (contrast_ratio(hex(a), hex(b)) * 100.0).round() / 100.0;
        assert_eq!(r("#000000", "#ffffff"), 21.0);
        assert_eq!(r("#ffffff", "#000000"), 21.0);
        assert_eq!(r("#777777", "#777777"), 1.0);
        // #767676 is the lightest grey that passes AA on white.
        assert_eq!(r("#767676", "#ffffff"), 4.54);
        assert_eq!(r("#777777", "#ffffff"), 4.48);
        // #595959 is the lightest grey that passes AAA on white.
        assert_eq!(r("#595959", "#ffffff"), 7.0);
        assert_eq!(r("#0000ff", "#ffffff"), 8.59);
        assert_eq!(r("#ff0000", "#ffffff"), 4.0);
    }

    // Values from the APCA reference implementation (apca-w3 0.1.9).
    #[test]
    fn apca_reference_values() {
        let lc = |t, b| (apca_lc(hex(t), hex(b)) * 10.0).round() / 10.0;
        assert_eq!(lc("#000000", "#ffffff"), 106.0);
        assert_eq!(lc("#ffffff", "#000000"), -107.9);
        assert_eq!(lc("#888888", "#ffffff"), 63.1);
        assert_eq!(lc("#ffffff", "#888888"), -68.5);
        assert_eq!(lc("#777777", "#777777"), 0.0);
    }

    #[test]
    fn composites_alpha() {
        let half_black = hex("#00000080");
        assert_eq!(half_black.over(hex("#ffffff")), hex("#7f7f7f"));
    }

    #[test]
    fn colour_blind_simulation_merges_red_and_green() {
        let (red, green) = (hex("#d62728"), hex("#2ca02c"));
        let normal = distance(red, green, Vision::Normal);
        let deutan = distance(red, green, Vision::Deuteranopia);
        assert!(deutan < normal / 2.0, "{deutan} vs {normal}");
        // Black and white stay apart for everyone.
        for v in Vision::ALL {
            assert!(distance(hex("#000"), hex("#fff"), v) > 0.99);
        }
    }
}
