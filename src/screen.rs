use std::{
	collections::VecDeque,
	fmt::Display,
	io::{self, stdout, StdoutLock, Write},
};

use termion::{
	self,
	raw::{IntoRawMode, RawTerminal},
	screen::AlternateScreen,
};

use crate::{dimension::Dimension, position::Position, component::Component};

pub struct Screen {
	screen: AlternateScreen<RawTerminal<StdoutLock<'static>>>,
	last_buffer: Option<ScreenBuffer>,
}

impl Screen {
	pub fn new() -> io::Result<Screen> {
		let stdout = stdout().lock();
		let raw_stdout = stdout.into_raw_mode()?;
		let mut screen = AlternateScreen::from(raw_stdout);
		write!(
			screen,
			"{}{}{}",
			termion::cursor::Save,
			termion::cursor::Hide,
			termion::clear::All
		)?;
		screen.flush()?;

		Ok(Screen {
			screen,
			last_buffer: None,
		})
	}

	pub fn draw(&mut self, root: Box<dyn Component>) -> io::Result<()> {
		let (width, height) = termion::terminal_size()?;
		let mut buffer = ScreenBuffer::new(Dimension { width, height });
		root.draw(&mut buffer.as_portion());

		let mut draw_commands: VecDeque<(Position, CellKind)> = VecDeque::new();

		match &self.last_buffer {
			Some(last_buffer) if last_buffer.dim == buffer.dim => {
				for y in 0..buffer.dim.height {
					for x in 0..buffer.dim.width {
						let pos = Position { x, y };
	
						if let Some(new_kind) = buffer.get(pos) {
							let changed_test = |kind| kind != new_kind;
							let pos_changed = last_buffer.get(pos).map(changed_test).unwrap_or(true);
	
							if pos_changed {
								let command = (Position { x: x + 1, y: y + 1 }, new_kind);
								draw_commands.push_back(command)
							}
						}
					}
				}
			},
			_ => {
				for y in 0..buffer.dim.height {
					for x in 0..buffer.dim.width {
						let index: usize = (x + y * buffer.dim.width).into();
						let command = (Position { x: x + 1, y: y + 1 }, buffer.cells[index]);
						draw_commands.push_back(command);
					}
				}
			},
		}

		if let Some((pos, kind)) = draw_commands.pop_front() {
			let mut output = String::new();
			output += &format!("{}{}", termion::cursor::Goto::from(pos), kind);
			let mut last_pos = pos;

			for (pos, kind) in draw_commands.into_iter() {
				if last_pos.x + 1 == pos.x {
					output += &format!("{}", kind);
				} else {
					output += &format!("{}{}", termion::cursor::Goto::from(pos), kind);
				}
				last_pos = pos;
			}

			write!(self.screen, "{}", output)?;
		}

		self.screen.flush()?;

		self.last_buffer = Some(buffer);

		Ok(())
	}
}

impl Drop for Screen {
	fn drop(&mut self) {
		write!(
			self.screen,
			"{}{}",
			termion::cursor::Show,
			termion::cursor::Restore
		)
		.unwrap()
	}
}

pub struct ScreenBuffer {
	dim: Dimension,
	cells: Vec<CellKind>,
}

impl ScreenBuffer {
	fn new(dim: Dimension) -> ScreenBuffer {
		let w = usize::from(dim.width);
		let h = usize::from(dim.height);
		let cells = vec![CellKind::Empty; w * h];

		ScreenBuffer { dim, cells }
	}

	fn as_portion(&mut self) -> ScreenPortion {
		ScreenPortion {
			dim: self.dim,
			offset_x: 0,
			offset_y: 0,
			buffer: self,
		}
	}

	fn get(&self, pos: Position) -> Option<CellKind> {
		let index: usize = (pos.x + pos.y * self.dim.width).into();
		self.cells.get(index).copied()
	}
}

pub struct ScreenPortion<'buffer> {
	dim: Dimension,
	offset_x: u16,
	offset_y: u16,
	buffer: &'buffer mut ScreenBuffer,
}

impl<'buffer> ScreenPortion<'buffer> {
	pub fn set(&mut self, x: u16, y: u16, kind: CellKind) -> Result<(), ScreenPortionSetError> {
		let ok_x = x < self.dim.width;
		let ok_y = y < self.dim.height;

		if !(ok_x && ok_y) {
			return Err(ScreenPortionSetError::OutOfBounds);
		}

		let absolute_x = x + self.offset_x;
		let absolute_y = y + self.offset_y;

		let buffer_index = usize::from(absolute_x + absolute_y * self.buffer.dim.width);

		let cell = self
			.buffer
			.cells
			.get_mut(buffer_index)
			.ok_or(ScreenPortionSetError::OutOfBounds)?;
		*cell = kind;

		Ok(())
	}

	pub fn dimension(&self) -> Dimension {
		self.dim
	}

	pub fn portion(
		&mut self,
		offset_x: u16,
		offset_y: u16,
		dim: Dimension,
	) -> Result<ScreenPortion, ScreenPortionPortionError> {
		let width_ok = offset_x + dim.width <= self.dim.width;
		let height_ok = offset_y + dim.height <= self.dim.height;
		if !(width_ok && height_ok) {
			return Err(ScreenPortionPortionError::OutOfBounds);
		}

		let absolute_offset_x = self.offset_x + offset_x;
		let absolute_offset_y = self.offset_y + offset_y;
		let buffer: &mut ScreenBuffer = self.buffer;

		Ok(ScreenPortion {
			dim,
			offset_x: absolute_offset_x,
			offset_y: absolute_offset_y,
			buffer,
		})
	}
}

#[derive(Debug)]
pub enum ScreenPortionSetError {
	OutOfBounds,
}

#[derive(Debug)]
pub enum ScreenPortionPortionError {
	OutOfBounds,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellKind {
	Empty,
	Char(Char),
	BoxBorder(BoxBorder),
}

impl From<Char> for CellKind {
	fn from(c: Char) -> Self {
		CellKind::Char(c)
	}
}

impl From<BoxBorder> for CellKind {
	fn from(b: BoxBorder) -> Self {
		CellKind::BoxBorder(b)
	}
}

impl Display for CellKind {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			CellKind::Empty => write!(f, " "),
			CellKind::Char(c) => write!(f, "{}", c),
			CellKind::BoxBorder(b) => write!(f, "{}", b),
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoxBorder {
	Right,
	Left,
	Top,
	Bottom,
	TopRight,
	TopLeft,
	BottomRight,
	BottomLeft,
}

impl Display for BoxBorder {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let c = match self {
			BoxBorder::Right => '│',
			BoxBorder::Left => '│',
			BoxBorder::Top => '─',
			BoxBorder::Bottom => '─',
			BoxBorder::TopRight => '┐',
			BoxBorder::TopLeft => '┌',
			BoxBorder::BottomRight => '┘',
			BoxBorder::BottomLeft => '└',
		};

		write!(f, "{}", c)
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Char {
	A(CharVariant),
	B(CharVariant),
	C(CharVariant),
	D(CharVariant),
	E(CharVariant),
	F(CharVariant),
	G(CharVariant),
	H(CharVariant),
	I(CharVariant),
	J(CharVariant),
	K(CharVariant),
	L(CharVariant),
	M(CharVariant),
	N(CharVariant),
	O(CharVariant),
	P(CharVariant),
	Q(CharVariant),
	R(CharVariant),
	S(CharVariant),
	T(CharVariant),
	U(CharVariant),
	V(CharVariant),
	W(CharVariant),
	X(CharVariant),
	Y(CharVariant),
	Z(CharVariant),
	Zero,
	One,
	Two,
	Three,
	Four,
	Five,
	Six,
	Seven,
	Eight,
	Nine,
	Unsupported,
}

impl Display for Char {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let c = match self {
			Char::A(v) => match v {
				CharVariant::Uppercase => 'A',
				CharVariant::Lowercase => 'a',
			},
			Char::B(v) => match v {
				CharVariant::Uppercase => 'B',
				CharVariant::Lowercase => 'b',
			},
			Char::C(v) => match v {
				CharVariant::Uppercase => 'C',
				CharVariant::Lowercase => 'c',
			},
			Char::D(v) => match v {
				CharVariant::Uppercase => 'D',
				CharVariant::Lowercase => 'd',
			},
			Char::E(v) => match v {
				CharVariant::Uppercase => 'E',
				CharVariant::Lowercase => 'e',
			},
			Char::F(v) => match v {
				CharVariant::Uppercase => 'F',
				CharVariant::Lowercase => 'f',
			},
			Char::G(v) => match v {
				CharVariant::Uppercase => 'G',
				CharVariant::Lowercase => 'g',
			},
			Char::H(v) => match v {
				CharVariant::Uppercase => 'H',
				CharVariant::Lowercase => 'h',
			},
			Char::I(v) => match v {
				CharVariant::Uppercase => 'I',
				CharVariant::Lowercase => 'i',
			},
			Char::J(v) => match v {
				CharVariant::Uppercase => 'J',
				CharVariant::Lowercase => 'j',
			},
			Char::K(v) => match v {
				CharVariant::Uppercase => 'K',
				CharVariant::Lowercase => 'k',
			},
			Char::L(v) => match v {
				CharVariant::Uppercase => 'L',
				CharVariant::Lowercase => 'l',
			},
			Char::M(v) => match v {
				CharVariant::Uppercase => 'M',
				CharVariant::Lowercase => 'm',
			},
			Char::N(v) => match v {
				CharVariant::Uppercase => 'N',
				CharVariant::Lowercase => 'n',
			},
			Char::O(v) => match v {
				CharVariant::Uppercase => 'O',
				CharVariant::Lowercase => 'o',
			},
			Char::P(v) => match v {
				CharVariant::Uppercase => 'P',
				CharVariant::Lowercase => 'p',
			},
			Char::Q(v) => match v {
				CharVariant::Uppercase => 'Q',
				CharVariant::Lowercase => 'q',
			},
			Char::R(v) => match v {
				CharVariant::Uppercase => 'R',
				CharVariant::Lowercase => 'r',
			},
			Char::S(v) => match v {
				CharVariant::Uppercase => 'S',
				CharVariant::Lowercase => 's',
			},
			Char::T(v) => match v {
				CharVariant::Uppercase => 'T',
				CharVariant::Lowercase => 't',
			},
			Char::U(v) => match v {
				CharVariant::Uppercase => 'U',
				CharVariant::Lowercase => 'u',
			},
			Char::V(v) => match v {
				CharVariant::Uppercase => 'V',
				CharVariant::Lowercase => 'v',
			},
			Char::W(v) => match v {
				CharVariant::Uppercase => 'W',
				CharVariant::Lowercase => 'w',
			},
			Char::X(v) => match v {
				CharVariant::Uppercase => 'X',
				CharVariant::Lowercase => 'x',
			},
			Char::Y(v) => match v {
				CharVariant::Uppercase => 'Y',
				CharVariant::Lowercase => 'y',
			},
			Char::Z(v) => match v {
				CharVariant::Uppercase => 'Z',
				CharVariant::Lowercase => 'z',
			},
			Char::Zero => '0',
			Char::One => '1',
			Char::Two => '2',
			Char::Three => '3',
			Char::Four => '4',
			Char::Five => '5',
			Char::Six => '6',
			Char::Seven => '7',
			Char::Eight => '8',
			Char::Nine => '9',
			Char::Unsupported => '?',
		};

		write!(f, "{}", c)
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharVariant {
	Uppercase,
	Lowercase,
}

impl From<char> for Char {
	fn from(c: char) -> Self {
		match c {
			'A' => Char::A(CharVariant::Uppercase),
			'a' => Char::A(CharVariant::Lowercase),
			'B' => Char::B(CharVariant::Uppercase),
			'b' => Char::B(CharVariant::Lowercase),
			'C' => Char::C(CharVariant::Uppercase),
			'c' => Char::C(CharVariant::Lowercase),
			'D' => Char::D(CharVariant::Uppercase),
			'd' => Char::D(CharVariant::Lowercase),
			'E' => Char::E(CharVariant::Uppercase),
			'e' => Char::E(CharVariant::Lowercase),
			'F' => Char::F(CharVariant::Uppercase),
			'f' => Char::F(CharVariant::Lowercase),
			'G' => Char::G(CharVariant::Uppercase),
			'g' => Char::G(CharVariant::Lowercase),
			'H' => Char::H(CharVariant::Uppercase),
			'h' => Char::H(CharVariant::Lowercase),
			'I' => Char::I(CharVariant::Uppercase),
			'i' => Char::I(CharVariant::Lowercase),
			'J' => Char::J(CharVariant::Uppercase),
			'j' => Char::J(CharVariant::Lowercase),
			'K' => Char::K(CharVariant::Uppercase),
			'k' => Char::K(CharVariant::Lowercase),
			'L' => Char::L(CharVariant::Uppercase),
			'l' => Char::L(CharVariant::Lowercase),
			'M' => Char::M(CharVariant::Uppercase),
			'm' => Char::M(CharVariant::Lowercase),
			'N' => Char::N(CharVariant::Uppercase),
			'n' => Char::N(CharVariant::Lowercase),
			'O' => Char::O(CharVariant::Uppercase),
			'o' => Char::O(CharVariant::Lowercase),
			'P' => Char::P(CharVariant::Uppercase),
			'p' => Char::P(CharVariant::Lowercase),
			'Q' => Char::Q(CharVariant::Uppercase),
			'q' => Char::Q(CharVariant::Lowercase),
			'R' => Char::R(CharVariant::Uppercase),
			'r' => Char::R(CharVariant::Lowercase),
			'S' => Char::S(CharVariant::Uppercase),
			's' => Char::S(CharVariant::Lowercase),
			'T' => Char::T(CharVariant::Uppercase),
			't' => Char::T(CharVariant::Lowercase),
			'U' => Char::U(CharVariant::Uppercase),
			'u' => Char::U(CharVariant::Lowercase),
			'V' => Char::V(CharVariant::Uppercase),
			'v' => Char::V(CharVariant::Lowercase),
			'W' => Char::W(CharVariant::Uppercase),
			'w' => Char::W(CharVariant::Lowercase),
			'X' => Char::X(CharVariant::Uppercase),
			'x' => Char::X(CharVariant::Lowercase),
			'Y' => Char::Y(CharVariant::Uppercase),
			'y' => Char::Y(CharVariant::Lowercase),
			'Z' => Char::Z(CharVariant::Uppercase),
			'z' => Char::Z(CharVariant::Lowercase),
			'0' => Char::Zero,
			'1' => Char::One,
			'2' => Char::Two,
			'3' => Char::Three,
			'4' => Char::Four,
			'5' => Char::Five,
			'6' => Char::Six,
			'7' => Char::Seven,
			'8' => Char::Eight,
			'9' => Char::Nine,
			_ => Char::Unsupported,
		}
	}
}
