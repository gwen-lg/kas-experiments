use kas::widgets::dialog::MessageBox;

fn main() -> Result<(), Box<dyn std::error::Error>> {
	env_logger::init();

	let window = MessageBox::new("Message", "Hello world");
	let theme = kas::theme::FlatTheme::new();
	kas::shell::DefaultShell::new(theme)?.with(window)?.run()
}
