use std::cell::RefCell;
use std::io;
use std::rc::Rc;
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
use super::cursor::PointerInfo;
use super::frame::XdgWindowFrame;

pub struct XdgWindowInner {
    pub globals: GlobalsHandle<XdgGlobals>,
    pub xdg_surface: Main<XdgSurface>,
    pub xdg_toplevel: Main<XdgToplevel>,
    pointer: PointerInfo,
    pub window: RefCell<WindowCommon>,
    frame: RefCell<XdgWindowFrame>,
}

impl Drop for XdgWindowInner {
    fn drop(&mut self) {
        self.xdg_toplevel.destroy();
        self.xdg_surface.destroy();
        let window = self.window.borrow_mut();
        window.surface.destroy();
    }
}

#[derive(Clone)]
pub struct XdgWindow {
    inner: Rc<XdgWindowInner>,
}

impl WindowBackend for XdgWindow {
    type BackendGlobals = XdgGlobals;
    fn new(globals: GlobalsHandle<Self::BackendGlobals>) -> Self {
        let window = WindowCommon::new(globals.clone());
        let xdg_globals = &globals.backend;
        let surface = window.surface.detach();
        let xdg_surface = xdg_globals.wm_base.get_xdg_surface(&surface);
        surface.commit();

        let xdg_toplevel = xdg_surface.get_toplevel();
        surface.commit();

        let frame = XdgWindowFrame::new(&surface, 40, globals.clone());
        surface.commit();

        globals.display.flush().unwrap();

        let inner = XdgWindowInner {
            xdg_surface,
            xdg_toplevel,
            frame: RefCell::new(frame),
            window: window.into(),
            pointer: PointerInfo::new("left_ptr", surface, globals.clone()),
            globals,
        };
        let inner = Rc::new(inner);
        let win = Self { inner };
        win.register_callbacks();
        win
    }

    fn window_common(&self) -> WindowCommon {
        self.inner.window.borrow().clone()
    }
}

impl XdgWindow {
    fn register_callbacks(&self) {
        let this = &self.inner;
        this.xdg_toplevel.quick_assign({
            let this = self.clone();
            move |_toplevel: Main<XdgToplevel>, evt: XdgToplevelEvent, _globals: _| match evt {
                XdgToplevelEvent::Configure {
                    mut width,
                    mut height,
                    states: _,
                } => {
                    if width == 0 || height == 0 {
                        width = 320;
                        height = 320;
                    }
                    this.inner.window.borrow_mut().resize(width, height);
                    this.inner.frame.borrow_mut().move_(width, height);

                    this.inner
                        .xdg_surface
                        .set_window_geometry(0, 0, width, height)
                }
                XdgToplevelEvent::Close => {
                    this.inner.window.borrow_mut().should_close = true;
                }
                e => println!("toplevel evt: {:?}", e),
            }
        });

        this.xdg_surface.quick_assign({
            let this = self.clone();
            move |surface: Main<XdgSurface>, evt: XdgSurfaceEvent, _| match evt {
                XdgSurfaceEvent::Configure { serial } => {
                    surface.ack_configure(serial);
                    this.inner.pointer.update_serial(serial);
                }
                _ => (),
            }
        });
    }
}
