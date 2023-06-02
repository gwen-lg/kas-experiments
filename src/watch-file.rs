use std::path::Path;
use std::thread;

use kas::event::UpdateId;
use kas::prelude::*;
use kas::widgets::dialog::Window;
use kas::widgets::ScrollLabel;
use notify::EventKind;

use futures::{
	channel::mpsc::{channel, Receiver},
	SinkExt, StreamExt,
};
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};

#[derive(Clone, Debug)]
enum FileWatch {
	Changed,
}

impl_scope! {
	#[widget{
		layout = column: [
			self.display,
		];
	}]
	#[impl_default]
	#[derive(Debug)]
	struct FileDisplay {
		core: widget_core!(),
		#[widget] display: ScrollLabel<String>,
		update_id: UpdateId = UpdateId::new(),
		loading_text: Text<&'static str>,
		//file: Calculator = Calculator::new(),
		loaded: bool = false,
	}
	impl FileDisplay {
		fn new(update_id: UpdateId) -> Self {
			FileDisplay {
				core: Default::default(),
				display:ScrollLabel::new("Test".to_string()),
				//.with_editable(false)
				// .with_multi_line(true),
				// .with_lines(3, 3)
				// .with_width_em(5.0, 10.0),
				update_id,
				loading_text: Text::new("Loading..."),
				loaded: false,
			}
		}
	}

	impl Widget for Self {
		fn handle_event(&mut self, mgr: &mut EventMgr, event: Event) -> Response {
			match event {
				Event::Update { id, .. } if id == self.update_id => {
					mgr.push(FileWatch::Changed);
					*mgr |= self.display.set_string("content changed".to_string());
					mgr.redraw(self.id());
					Response::Used
				}
				_ => Response::Unused,
			}
		}
		// fn handle_message(&mut self, mgr: &mut EventMgr) {
		// 	println!("message");
		// 	if let Some(msg) = mgr.try_pop::<FileWatch>() {
		// 		match msg {
		// 			FileWatch::Changed => {
		// 				*mgr |= self.display.set_string("content changed".to_string());
		// 				self.loaded = true;
		// 			}
		// 		}
		// 	}
		// }
	}
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
	env_logger::init();

	let theme = kas_wgpu::ShadedTheme::new();
	let shell = kas::shell::DefaultShell::new(theme)?;
	// We construct a proxy from the shell to enable cross-thread communication.
	let proxy = shell.create_proxy();

	let update_id = UpdateId::new();
	watch_file_update(proxy, update_id);

	let file_display = FileDisplay::new(update_id);
	let window = Window::new("File watcher", file_display);
	shell.with(window)?.run()
}

fn watch_file_update(proxy: kas::shell::Proxy, update_id: UpdateId) {
	const FILENAME: &str = "file_watched.txt";

	thread::Builder::new()
		.name("Watcher".to_string())
		.spawn(move || {
			futures::executor::block_on(async {
				if let Err(e) = async_watch(proxy, update_id, FILENAME).await {
					println!("error: {:?}", e)
				}
			});
		})
		.unwrap();
}

fn async_watcher() -> notify::Result<(RecommendedWatcher, Receiver<notify::Result<notify::Event>>)>
{
	let (mut tx, rx) = channel(1);

	// Automatically select the best implementation for your platform.
	// You can also access each implementation directly e.g. INotifyWatcher.
	let watcher = RecommendedWatcher::new(
		move |res| {
			futures::executor::block_on(async {
				//let msg = FileWatch::Changed;
				tx.send(res).await.unwrap();
			})
		},
		Config::default(),
	)?;

	Ok((watcher, rx))
}

async fn async_watch<P: AsRef<Path>>(
	proxy: kas::shell::Proxy,
	update_id: UpdateId,
	path: P,
) -> notify::Result<()> {
	let (mut watcher, mut rx) = async_watcher()?;

	// Add a path to be watched. All files and directories at that path and
	// below will be monitored for changes.
	watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;

	println!("watching {}", path.as_ref().display());

	while let Some(res) = rx.next().await {
		match res {
			Ok(event) => {
				match event.kind {
					EventKind::Modify(_) => {
						proxy.update_all(update_id, 0).unwrap_or(());
					} //HACK
					_ => (),
				}
			}
			Err(e) => println!("watch error: {:?}", e),
		}
	}

	Ok(())
}
