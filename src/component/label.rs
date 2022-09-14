use crate::screen::{ScreenPortion, Char, CellKind};

use super::Component;

pub struct Label {
	pub text: String,
}

impl Component for Label {
	fn draw(&self, buffer: &mut ScreenPortion) {
		let kinds = self
			.text
			.chars()
			.map(Char::from)
			.map(CellKind::from)
			.take(buffer.dimension().width.into());

		let mut x = 0u16;
		for kind in kinds {
			buffer.set(x, 0, kind).unwrap();
			x += 1;
		}
	}
}