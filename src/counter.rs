use kas::model::SharedRc;
use kas::prelude::*;
use kas::view::SingleView;
use kas::widgets::TextButton;

#[derive(Clone, Debug)]
struct Increment(i32);

impl_scope! {
	// We add the `#[widget]` attribute and use it to specify layout:
	#[widget{
		layout = column: [
			align(center): self.display,
			row: [
				TextButton::new_msg("−", Increment(-1)),
				TextButton::new_msg("+", Increment(1))
			],
		];
	}]
	#[derive(Debug, Clone)]
	struct Counter {
		core: widget_core!(),
		#[widget] display: SingleView<SharedRc<i32>>,
	}

	// `impl Self` is equivalent to `impl Counter` here.
	// It's more useful when the type has generic parameters!
	impl Self {
		fn new(count: i32) -> Self {
			Counter {
				core: Default::default(),
				display: SingleView::new(SharedRc::new(count)),
			}
		}
	}
	impl Widget for Self {
		fn handle_message(&mut self, mgr: &mut EventMgr) {
			if let Some(Increment(incr)) = mgr.try_pop() {
				self.display.update_value(mgr, |count| *count += incr);
			}
		}
	}
	impl Window for Counter {
		fn title(&self) -> &str { "Counter" }
	}
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
	env_logger::init();

	let theme = kas::theme::FlatTheme::new().with_font_size(24.0);

	let counter = Counter::new(0);
	kas::shell::DefaultShell::new(theme)?
		.with(counter.clone())?
		.with(counter)?
		.run()
}
