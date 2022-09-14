use crate::{screen::{ScreenPortion, BoxBorder}, dimension::Dimension};

use super::Component;

pub struct Border<C> {
	pub c: C,
}

impl<C: Component> Component for Border<C> {
	fn draw(&self, buffer: &mut ScreenPortion) {
		if buffer.dimension().width < 2 || buffer.dimension().height < 2 {
			self.c.draw(buffer);
			return;
		}

		buffer.set(0, 0, BoxBorder::TopLeft.into()).unwrap();
		buffer
			.set(buffer.dimension().width - 1, 0, BoxBorder::TopRight.into())
			.unwrap();
		buffer
			.set(
				buffer.dimension().width - 1,
				buffer.dimension().height - 1,
				BoxBorder::BottomRight.into(),
			)
			.unwrap();
		buffer
			.set(
				0,
				buffer.dimension().height - 1,
				BoxBorder::BottomLeft.into(),
			)
			.unwrap();

		for x in 1..buffer.dimension().width - 1 {
			buffer.set(x, 0, BoxBorder::Top.into()).unwrap();

			buffer
				.set(x, buffer.dimension().height - 1, BoxBorder::Bottom.into())
				.unwrap();
		}

		for y in 1..buffer.dimension().height - 1 {
			buffer.set(0, y, BoxBorder::Left.into()).unwrap();

			buffer
				.set(buffer.dimension().width - 1, y, BoxBorder::Right.into())
				.unwrap();
		}

		let offset_x = 1;
		let offset_y = 1;
		let dim = Dimension {
			width: buffer.dimension().width - 2,
			height: buffer.dimension().height - 2,
		};

		let mut buffer = buffer.portion(offset_x, offset_y, dim).unwrap();

		self.c.draw(&mut buffer);
	}
}

pub trait WithBorder
where
	Self: Sized,
{
	fn with_border(self: Self) -> Border<Self>;
}

impl<C: Component> WithBorder for C {
	fn with_border(self: Self) -> Border<Self> {
		Border { c: self }
	}
}