#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
	pub x: u16,
	pub y: u16,
}

impl From<Position> for termion::cursor::Goto {
	fn from(p: Position) -> Self {
		termion::cursor::Goto(p.x, p.y)
	}
}