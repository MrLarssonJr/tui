#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Dimension {
	pub width: u16,
	pub height: u16,
}

impl From<(u16, u16)> for Dimension {
	fn from((height, width): (u16, u16)) -> Self {
		Dimension { width, height }
	}
}