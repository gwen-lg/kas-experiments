use kas::model::SharedRc;
use kas::view::{driver, SingleView};
use kas::widgets::dialog::Window;

fn main() -> kas::shell::Result<()> {
	env_logger::init();

	let driver = driver::Spinner::new(i32::MIN..=10, 1);
	let c1 = SingleView::new_with_driver(driver, SharedRc::new(0));
	let c2 = SingleView::new_with_driver(driver, c1.data().clone());

	let theme = kas_wgpu::ShadedTheme::new().with_font_size(24.0);
	kas::shell::DefaultShell::new(theme)?
		.with(Window::new("Counter 1", c1))?
		.with(Window::new("Counter 2", c2))?
		.run()
}
