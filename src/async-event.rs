use std::{
	sync::{Arc, Mutex},
	thread,
	time::{Duration, Instant},
};

use kas::{draw::color::Rgba, event::UpdateId};
use kas::{prelude::*, theme::TextClass};

fn main() -> kas::shell::Result<()> {
	env_logger::init();

	let theme = kas::theme::FlatTheme::new();
	let shell = kas::shell::DefaultShell::new(theme)?;

	// We construct a proxy from the shell to enable cross-thread communication.
	let proxy = shell.create_proxy();

	// UpdateId is used to identify the source of Event::Update (not essential in this example).
	let update_id = UpdateId::new();

	// The sender and receiver need to communicate. We use Arc<Mutex<T>>, but
	// could instead use global statics or std::sync::mpsc or even encode our
	// data within the update payload (a u64, so some compression required).
	let colour = Arc::new(Mutex::new(Rgba::grey(1.0)));
	let colour2 = colour.clone();

	thread::spawn(move || generate_colors(proxy, update_id, colour2));

	let widget = ColourSquare::new(colour, update_id);

	shell.with(widget)?.run()
}

impl_scope! {
	#[derive(Debug)]
	#[widget]
	struct ColourSquare {
		core: widget_core!(),
		colour: Arc<Mutex<Rgba>>,
		update_id: UpdateId,
		loading_text: Text<&'static str>,
		loaded: bool,
	}
	impl Self {
		fn new(colour: Arc<Mutex<Rgba>>, update_id: UpdateId) -> Self {
			ColourSquare {
				core: Default::default(),
				colour,
				update_id,
				loading_text: Text::new("Loading..."),
				loaded: false,
			}
		}
	}
	impl Layout for ColourSquare {
		fn size_rules(&mut self, mgr: SizeMgr, _: AxisInfo) -> SizeRules {
			SizeRules::fixed_scaled(100.0, 10.0, mgr.scale_factor())
		}

		fn set_rect(&mut self, mgr: &mut ConfigMgr, rect: Rect) {
			self.core.rect = rect;
			let align = Some(AlignPair::new(Align::Center, Align::Center));
			mgr.text_set_size(
				&mut self.loading_text,
				TextClass::Label(false),
				rect.size,
				align,
			);
		}

		fn draw(&mut self, mut draw: DrawMgr) {
			if !self.loaded {
				draw.text(self.core.rect, &self.loading_text, TextClass::Label(false));
			} else {
				let draw = draw.draw_device();
				let col = *self.colour.lock().unwrap();
				draw.rect((self.rect()).cast(), col);
			}
		}
	}
	impl Widget for ColourSquare {
		fn handle_event(&mut self, mgr: &mut EventMgr, event: Event) -> Response {
			match event {
				Event::Update { id, .. } if id == self.update_id => {
					self.loaded = true;
					mgr.redraw(self.id());
					Response::Used
				}
				_ => Response::Unused,
			}
		}
	}
	impl Window for Self {
		fn title(&self) -> &str {
			"Async event demo"
		}
	}
}

fn generate_colors(proxy: kas::shell::Proxy, update_id: UpdateId, colour: Arc<Mutex<Rgba>>) {
	// Loading takes time:
	thread::sleep(Duration::from_secs(1));

	// This function is called in a separate thread, and runs until the program ends.
	let start_time = Instant::now();

	loop {
		let hue = (Instant::now() - start_time).as_secs_f32() / 5.0;

		// convert from HSV, using S=V=1 (see Wikipedia):
		let f = |n| {
			let k: f32 = (n + hue * 6.0) % 6.0;
			1.0 - k.min(4.0 - k).clamp(0.0, 1.0)
		};
		let c = Rgba::rgb(f(5.0), f(3.0), f(1.0));

		// Communicate the colour ...
		*colour.lock().unwrap() = c;
		// .. and notify of an update.
		// (Note: the 0 here is the u64 payload, which could pass useful data!)
		if proxy.update_all(update_id, 0).is_err() {
			// Sending failed; we should quit
			break;
		}

		thread::sleep(Duration::from_millis(20));
	}
}
