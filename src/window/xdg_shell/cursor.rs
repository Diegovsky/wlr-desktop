use std::sync::atomic::AtomicU32;

use smithay_client_toolkit::shm::AutoMemPool;
use wayland_client::protocol::wl_pointer::{self, WlPointer};
use wayland_client::protocol::wl_shm;
use wayland_client::protocol::wl_subsurface::WlSubsurface;
use wayland_client::protocol::wl_surface::WlSurface;
use wayland_client::{Main, Proxy, ProxyMap};
use wayland_cursor::CursorTheme;
use wayland_protocols::xdg_shell::client::xdg_wm_base;
use wayland_protocols::xdg_shell::client::{
    xdg_surface::{Event as XdgSurfaceEvent, XdgSurface},
    xdg_toplevel::{Event as XdgToplevelEvent, XdgToplevel},
    xdg_wm_base::XdgWmBase,
};

use crate::prelude::{GlobalsHandle, RcCell};
use crate::window::{WindowBackend, WindowCommon};

use super::globals::XdgGlobals;

#[derive(Debug)]
struct PointerInfoInner {
    wl_surface: WlSurface,
    cursor_surface: Main<WlSurface>,
    cursor_pos: (f64, f64),
    serial: u32,
    is_inside: bool,
}

impl PointerInfoInner {
    fn handle_pointer_event(&mut self, evt: wl_pointer::Event, ptr: Main<WlPointer>) {
        match evt {
            wl_pointer::Event::Enter {
                serial,
                surface,
                surface_x: x,
                surface_y: y,
            } => {
                if !self.is_inside && surface == self.wl_surface {
                    self.is_inside = true;
                    self.update_pointer_pos(x, y);
                    self.serial = serial;
                    ptr.set_cursor(self.serial, Some(&self.cursor_surface), 0, 0);
                }
            }
            wl_pointer::Event::Motion {
                surface_x: x,
                surface_y: y,
                ..
            } => {
                if self.is_inside {
                    self.update_pointer_pos(x, y);
                }
            }, 
            wl_pointer::Event::Leave {serial, surface } => {
                if self.is_inside && surface == self.wl_surface {
                    self.is_inside = false;
                    self.serial = serial;
                    ptr.set_cursor(self.serial, None, 0, 0);
                }
            }
            _ => (),
        }
    }
    fn update_pointer_pos(&mut self, x: f64, y: f64) {
        self.cursor_pos = (x, y);
    }
}

#[derive(Debug, Clone)]
pub struct PointerInfo {
    inner: RcCell<PointerInfoInner>,
}

impl PointerInfo {
    pub fn new(cursor_name: &str, wl_surface: WlSurface, globals: GlobalsHandle<XdgGlobals>) -> Self {
        let Some(cursor_surface) = globals.get_cursor(cursor_name) else {
            panic!("Cursor surface {} not found", cursor_name);
        };
        let this = Self {
            inner: PointerInfoInner {
                wl_surface,
                cursor_surface,
                cursor_pos: (0.0, 0.0),
                serial: 0,
                is_inside: false,
            }
            .into(),
        };
        globals.wl_pointer.quick_assign({ let this = this.clone(); move |ptr, evt, _| this.handle_pointer_event(evt, ptr) });
        this
    }
    pub fn cursor_pos(&self) -> (f64, f64) {
        self.inner.borrow().cursor_pos
    }
    pub fn serial(&self) -> u32 {
        self.inner.borrow().serial
    }
    pub fn is_inside(&self) -> bool {
        self.inner.borrow().is_inside
    }

    pub fn update_serial(&self, serial: u32) {
        self.inner.borrow_mut().serial = serial;
    }

    pub fn handle_pointer_event(&self, evt: wl_pointer::Event, ptr: Main<WlPointer>) {
        self.inner.borrow_mut().handle_pointer_event(evt, ptr);
    }
    pub fn update_pointer_pos(&self, x: f64, y: f64) {
        self.inner.borrow_mut().update_pointer_pos(x, y)
    }
}
