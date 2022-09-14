use std::{
	io,
	thread::JoinHandle,
};

use termion::input::TermRead;

use crate::{screen::Screen, component::Component};

fn run<A: App>() -> io::Result<()> {
	let mut state = A::State::default();
	let mut screen = Screen::new()?;

	let (tx, event_queue) = std::sync::mpsc::channel::<A::Event>();

	let _input_join_handle: JoinHandle<io::Result<()>> = std::thread::spawn(move || {
		let stdin = std::io::stdin().lock();
		let _event_queue = tx;

		for term_event in stdin.events() {
			let term_event = term_event?;
			match term_event {
				termion::event::Event::Key(termion::event::Key::Ctrl('c')) => break,
				_ => (),
			}
		}

		Ok(())
	});

	screen.draw(A::view(&state))?;

	for event in event_queue.iter() {
		state = A::update(state, event);
		let view = A::view(&state);
		screen.draw(view)?;
	}

	Ok(())
}

pub trait App: Sized {
	type Event: Send + 'static;
	type State: Default;

	fn update(state: Self::State, event: Self::Event) -> Self::State;

	fn view(state: &Self::State) -> Box<dyn Component>;

	fn run(self) {
		let _res = run::<Self>();
	}
}
