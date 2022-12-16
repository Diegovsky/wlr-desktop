use globals::GlobalsHandle;


use wayland_client::GlobalManager;
use window::{xdg_shell::{XdgGlobals}, WindowBackend};

mod renderer;
mod window;

mod nullable;
mod globals;
mod prelude;

use prelude::*;

fn main() {
    let display = wayland_client::Display::connect_to_env().expect("Wayland not found!");
    let mut queue = display.create_event_queue();
    queue.sync_roundtrip(&mut (), |_, _, _| {}).unwrap();
    
    let global_manager = GlobalManager::new(&display.attach(queue.token()));
    queue.sync_roundtrip(&mut (), |_, _, _| {}).unwrap();
    let globals = GlobalsHandle::<XdgGlobals>::new(global_manager);
    let window = globals.new_window();
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let mut st = String::new();
        let stdin = std::io::stdin();  
        loop {
            st.clear();
            let Ok(_) = stdin.read_line(&mut st) else { break };
            let pog: String = st.trim().into(); 
            tx.send(pog).unwrap();
        };
    });
    loop {
        if window.should_close() {
            break;
        }
        queue.dispatch(&mut (), |_, _, _| {}).unwrap();
        let Ok(c) = rx.try_recv() else { continue };

        let mut cursor_theme = globals.backend.cursor_theme.borrow_mut();
        let cursor_image = &cursor_theme.get_cursor(&c).unwrap()[0];
        window.attach_cursor(cursor_image);
    }
}
