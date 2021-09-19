#[macro_use]
extern crate cascade;

use async_channel::Sender;
use gtk::prelude::*;
use std::future::Future;
use std::process;

enum Event {
    Clicked,
}

struct App {
    pub button: gtk::Button,
    pub clicked: u32,
}

fn main() {
    glib::set_program_name("counter".into());
    glib::set_application_name("counter");

    if gtk::init().is_err() {
        eprintln!("failed to initialized the gtk application");
        process::exit(1);
    }

    //attach a tx (send) - to our widgets
    //and attach a rx (receive) - to our event handler
    let (tx, rx) = async_channel::unbounded();

    let mut app = App::new(tx);

    //process all applications events from signals
    let event_handler = async move {
        while let Ok(event) = rx.recv().await {
            match event {
                Event::Clicked => {
                    app.clicked += 1;
                    app.button.set_label(&format!(
                        "I have been clicked this many times -> {}",
                        app.clicked
                    ));
                }
            }
        }
    };

    //glib has an executor in the background that will asynchronously handle
    //our events on this thread
    glib::MainContext::default().spawn_local(event_handler);

    //thread will block here until application is quit
    gtk::main();
}

//spawns a feature on main thread, without waiting for it to complete
pub fn spawn<F>(future: F)
where
    F: Future<Output = ()> + 'static,
{
    glib::MainContext::default().spawn_local(future)
}

impl App {
    pub fn new(tx: Sender<Event>) -> Self {
        let button = cascade! {
            gtk::Button::with_label("click me");
            ..set_border_width(4);
            ..connect_clicked(move |_| {
                let tx = tx.clone();
                spawn(async move {
                    let _ = tx.send(Event::Clicked).await;
                });
            });
        };

        let container = cascade! {
            gtk::Box::new(gtk::Orientation::Vertical, 0);
            ..add(&button);
            ..show_all();
        };

        let _window = cascade! {
            gtk::Window::new(gtk::WindowType::Toplevel);
            ..add(&container);
            ..set_title("counter tutorial");
            ..set_default_size(400, 250);
            ..set_position(gtk::WindowPosition::CenterAlways);
            ..connect_delete_event(move |_,_| {
                gtk::main_quit();
                gtk::Inhibit(false)
            });
            ..show_all();
        };

        gtk::Window::set_default_icon_name("jaw dropping icon name here");

        Self { button, clicked: 0 }
    }
}
