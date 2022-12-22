use std::cell::RefCell;
use std::io;
use std::rc::Rc;
use std::sync::atomic::AtomicU32;

use smithay_client_toolkit::shm::AutoMemPool;
use wayland_client::protocol::wl_pointer::{self, WlPointer};
use wayland_client::protocol::wl_shm;
use wayland_client::protocol::wl_surface::WlSurface;
use wayland_client::protocol::wl_subsurface::WlSubsurface;
use wayland_client::{Main, Proxy, ProxyMap};
use wayland_cursor::CursorTheme;
use wayland_protocols::xdg_shell::client::xdg_wm_base;
use wayland_protocols::xdg_shell::client::{
    xdg_surface::{Event as XdgSurfaceEvent, XdgSurface},
    xdg_toplevel::{Event as XdgToplevelEvent, XdgToplevel},
    xdg_wm_base::XdgWmBase,
};

use crate::prelude::{GlobalsHandle, RcCell};
use crate::window::{WindowCommon, WindowBackend};

use super::globals::XdgGlobals;


pub struct XdgWindowInner {
    pub globals: GlobalsHandle<XdgGlobals>,
    pub xdg_surface: Main<XdgSurface>,
    pub xdg_toplevel: Main<XdgToplevel>,
    pub wl_pointer: Main<WlPointer>,
    pointer: RefCell<PointerInfo>,
    pub window: RefCell<WindowCommon>,
    frame: RefCell<XdgWindowFrame>,
}

#[derive(Debug)]
struct XdgWindowFrame {
    padding: i32,
    // laterals: [XdgWindowBorder; 4],
    corners: [XdgWindowBorder; 4],
}

impl XdgWindowFrame {
    fn new(parent: &WlSurface, padding: u32, globals: GlobalsHandle<XdgGlobals>) -> Self {
        let padding = padding as i32; 
        let make_border = |cursor_name| {
            XdgWindowBorder::new((0, 0), (padding, padding), cursor_name, parent, globals.clone()) 
        };
        let names = [
            "top_left_corner",
            "top_right_corner",
            "bottom_left_corner",
            "bottom_right_corner"
        ];

        // let laterals = todo!();
        let corners = names.map(|name| make_border(name));
        Self { corners, padding }
    }
    fn move_(&mut self, width: i32, height: i32) {
        let corners_pos = [
            [0, 0], // top-left
            [1, 0], // top-right
            [0, 1], // bottom-left
            [1, 1]  // bottom-right
        ];
        let corners_displacement = corners_pos.map(|pos| pos.map(|c| c-1));
        let corner_radius = 8;
        let parent_size = [width, height];
        let corners = {
            let mut pos = [[0i32; 2]; 4];
            for i in 0..4 {
                for j in 0..2 {
                    pos[i][j] = corners_pos[i][j]*parent_size[j]+corners_displacement[i][j]*self.padding;
                }
            }
            pos
        };
        for (i, x) in self.corners.iter_mut().enumerate() {
            x.move_(corners[i][0], corners[i][1]);
        }
    }
}

#[derive(Debug)]
struct XdgWindowBorder {
    pub pos: (i32, i32),
    pub size: (i32, i32),
    pointer_info: PointerInfo,
    pub wl_surface: Main<WlSurface>,
    pub wl_subsurface: Main<WlSubsurface>,
    shm_pool: RcCell<AutoMemPool>,
}

impl XdgWindowBorder {
    fn new(pos: (i32, i32), size: (i32, i32), cursor_name: &str, parent_surface: &WlSurface, globals: GlobalsHandle<XdgGlobals>) -> Self {
        let wl_surface = globals.wl_compositor.create_surface();
        let wl_subsurface = globals.wl_subcompositor.get_subsurface(&wl_surface.detach(), parent_surface);
        wl_subsurface.place_below(&parent_surface);
        wl_subsurface.set_position(pos.0, pos.1);
        let mut this = Self {
            pos,
            size,
            wl_subsurface,
            shm_pool: globals.shm_pool.clone(),
            pointer_info: PointerInfo::new(cursor_name, wl_surface.detach().into(), globals),
            wl_surface,
        };
        this.resize(size.0, size.1);
        this
    }
    fn move_(&mut self, x: i32, y: i32) {
        self.pos = (x, y);
        self.wl_subsurface.set_position(x, y);
    }
    fn resize(&mut self, width: i32, height: i32) {
        let mut shm_pool = self.shm_pool.borrow_mut();
        let (buf, wl_buf) = shm_pool.buffer(width, height, width*4, wl_shm::Format::Xrgb8888).expect("Failed to allocate memory");
        buf.fill(0xff);
        self.size = (width, height);
        self.wl_surface.damage_buffer(0, 0, width, height);
        self.wl_surface.attach(Some(&wl_buf), 0, 0);
        self.wl_surface.offset(0, 0);
        self.wl_surface.commit();
    }
}

#[derive(Clone, Debug)]
struct PointerInfo {
    wl_surface: WlSurface,
    cursor_surface: Main<WlSurface>,
    cursor_pos: (f64, f64),
    serial: u32,
    is_inside: bool,
}

impl PointerInfo {
    fn new(cursor_name: &str, wl_surface: WlSurface, globals: GlobalsHandle<XdgGlobals>) -> Self {
        let Some(cursor_surface) = globals.get_cursor(cursor_name) else {
            panic!("Cursor surface {} not found", cursor_name);
        };
        Self { wl_surface, cursor_surface, cursor_pos: (0.0,0.0), serial: 0, is_inside: false }
    }
    fn handle_pointer_event(&mut self, evt: wl_pointer::Event, ptr: Main<WlPointer>) {
        match evt {
            wl_pointer::Event::Enter {
                serial,
                surface,
                surface_x: x,
                surface_y: y,
            } => {
                if !self.is_inside {
                    self.is_inside = surface == self.wl_surface.clone().into();
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
            }
            _ => (),
        }
    }
    fn update_pointer_pos(&mut self, x: f64, y: f64) {
        self.cursor_pos = (x, y);
    }
}

impl Drop for XdgWindowInner {
    fn drop(&mut self) {
        self.wl_pointer.release();
        self.xdg_toplevel.destroy();
        self.xdg_surface.destroy();
        let window = self.window.borrow_mut();
        window.surface.destroy();
        if let Some(ref buf) = window.buffer {
            buf.destroy()
        }
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

        let frame = XdgWindowFrame::new(&surface, 20, globals.clone());
        surface.commit();

        globals.display.flush().unwrap();

        let inner = XdgWindowInner {
            wl_pointer: globals.wl_seat.get_pointer(),
            xdg_surface,
            xdg_toplevel,
            frame: RefCell::new(frame),
            window: window.into(),
            pointer: RefCell::new(PointerInfo::new("left_ptr", surface, globals.clone())),
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
                    this.inner.pointer.borrow_mut().serial = serial;
                }
                _ => (),
            }
        });

        this.wl_pointer.quick_assign({
            let this = self.clone();
            move |ptr, evt, _| {
                let pad = 10;
                let mut pointer_info = this.inner.pointer.borrow_mut(); 
                pointer_info.handle_pointer_event(evt, ptr)
            }
        })
    }
}
