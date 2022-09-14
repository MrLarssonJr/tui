use crate::{screen::ScreenPortion, dimension::Dimension};

use super::Component;

pub struct VerticalSplit<L, R> {
	pub left: L,
	pub right: R,
}

impl<L: Component, R: Component> Component for VerticalSplit<L, R> {
	fn draw(&self, buffer: &mut ScreenPortion) {
		let left_width = buffer.dimension().width / 2;
		let right_width = buffer.dimension().width - left_width;

		let left_dim = Dimension {
			width: left_width,
			height: buffer.dimension().height,
		};
		let right_dim = Dimension {
			width: right_width,
			height: buffer.dimension().height,
		};

		let mut left_buffer = buffer.portion(0, 0, left_dim).unwrap();
		self.left.draw(&mut left_buffer);

		let mut right_buffer = buffer.portion(left_width, 0, right_dim).unwrap();
		self.right.draw(&mut right_buffer)
	}
}