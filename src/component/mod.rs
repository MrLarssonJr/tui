mod label;
mod split;
mod border;

pub use label::Label;
pub use split::VerticalSplit;
pub use border::{Border, WithBorder};

use crate::screen::ScreenPortion;

pub trait Component {
	fn draw(&self, buffer: &mut ScreenPortion);

	fn boxed(self) -> Box<dyn Component> where Self: Sized + 'static {
		Box::new(self)
	}
}