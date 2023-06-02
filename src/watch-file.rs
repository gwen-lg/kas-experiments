use kas::widgets::dialog::Window;
use kas::widgets::ScrollLabel;

fn main() -> Result<(), Box<dyn std::error::Error>> {
	env_logger::init();

	let textarea = ScrollLabel::new("default");

	let window = Window::new("File watcher", textarea);
	let theme = kas_wgpu::ShadedTheme::new();
	kas::shell::DefaultShell::new(theme)?.with(window)?.run()
}
