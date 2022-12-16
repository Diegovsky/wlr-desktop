
use std::rc::Rc;

use wayland_client::protocol::wl_pointer::{self, WlPointer};
use wayland_client::protocol::wl_shm;
use wayland_client::protocol::wl_surface::WlSurface;
use wayland_client::Main;
use wayland_cursor::CursorTheme;
use wayland_protocols::xdg_shell::client::xdg_wm_base;
use wayland_protocols::xdg_shell::client::{
    xdg_surface::{Event as XdgSurfaceEvent, XdgSurface},
    xdg_toplevel::{Event as XdgToplevelEvent, XdgToplevel},
    xdg_wm_base::XdgWmBase,
};

use crate::{globals::BackendGlobals, prelude::RcCell, GlobalManagerExt, GlobalsHandle};

use super::window::XdgWindow;

pub struct XdgGlobals {
    pub wm_base: Main<XdgWmBase>,
    pub cursor_theme: RcCell<CursorTheme>,
}

impl GlobalsHandle<XdgGlobals> {
    pub fn get_cursor(&self, name: &str) -> Option<Main<WlSurface>> {
        let mut cursor_theme = self.backend.cursor_theme.borrow_mut();
        let cursor_image = &cursor_theme.get_cursor(name)?[0];
        let cursor_surface = self.wl_compositor.create_surface();
        let (width, height) = cursor_image.dimensions();
        cursor_surface.attach(Some(&cursor_image), 0, 0);
        cursor_surface.damage_buffer(0, 0, width as i32, height as i32);
        cursor_surface.commit();
        Some(cursor_surface)
    }
}


fn get_cursor_size() -> anyhow::Result<u32> {
    let var = std::env::var("XCURSOR_SIZE")?;
    Ok(var.parse()?)
}

impl BackendGlobals for XdgGlobals {
    type Window = XdgWindow;
    fn new(global_manager: &wayland_client::GlobalManager) -> Rc<Self> {
        let wl_shm = global_manager.get::<wl_shm::WlShm>();
        let xdg_globals = XdgGlobals {
            cursor_theme: CursorTheme::load(get_cursor_size().unwrap_or(24), &wl_shm).into(),
            wm_base: global_manager.get(),
        };
        xdg_globals.wm_base.quick_assign(
            |wm_base: Main<XdgWmBase>, event: xdg_wm_base::Event, _| match event {
                xdg_wm_base::Event::Ping { serial } => {
                    wm_base.pong(serial);
                }
                _ => (),
            },
        );
        Rc::new(xdg_globals)
    }
}
