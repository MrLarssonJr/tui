#![feature(generic_associated_types)]

use gat_pg::{component::*, App};

fn main() {
	Foo {}.run();
}

struct Foo {}

impl App for Foo {
	type Event = Event;

	type State = State;

	fn update(state: Self::State, event: Self::Event) -> Self::State {
		match event {
			Event::Tick => State {
				count: state.count + 1,
				..state
			},
		}
	}

	fn view(state: &Self::State) -> Box<dyn Component> {
		VerticalSplit {
			left: Label {
				text: state.count.to_string(),
			}
			.with_border(),
			right: Label {
				text: "Right".to_string(),
			}
			.with_border(),
		}.boxed()
	}
}

#[derive(Default, Debug)]
struct State {
	count: usize,
}

#[derive(Debug)]
enum Event {
	Tick,
}
