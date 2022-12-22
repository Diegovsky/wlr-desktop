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
    let globals = GlobalsHandle::<XdgGlobals>::new(global_manager, &display);
    let window = globals.new_window();
    loop {
        if window.should_close() {
            break;
        }
        queue.dispatch(&mut (), |_, _, _| {}).unwrap();
    }
}
