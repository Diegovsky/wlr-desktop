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

#[derive(Debug)]
pub struct XdgWindowFrame {
    padding: i32,
    // laterals: [XdgWindowBorder; 4],
    corners: [XdgWindowBorder; 4],
}

impl XdgWindowFrame {
    pub fn new(parent: &WlSurface, padding: u32, globals: GlobalsHandle<XdgGlobals>) -> Self {
        let padding = padding as i32;
        let make_border = |cursor_name| {
            XdgWindowBorder::new(
                (0, 0),
                (padding, padding),
                cursor_name,
                parent,
                globals.clone(),
            )
        };
        let names = [
            "top_left_corner",
            "top_right_corner",
            "bottom_left_corner",
            "bottom_right_corner",
        ];

        // let laterals = todo!();
        let corners = names.map(|name| make_border(name));
        Self { corners, padding }
    }
    pub fn move_(&mut self, width: i32, height: i32) {
        let corners_pos = [
            [0, 0], // top-left
            [1, 0], // top-right
            [0, 1], // bottom-left
            [1, 1], // bottom-right
        ];
        let border_len = self.padding;
        let parent_size = [width, height];
        let corners = {
            let mut pos = [[0i32; 2]; 4];
            for i in 0..4 {
                for j in 0..2 {
                    pos[i][j] = corners_pos[i][j] * parent_size[j] - (border_len / 2);
                }
            }
            pos
        };
        for (i, x) in self.corners.iter_mut().enumerate() {
            x.resize(border_len, border_len);
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
    fn new(
        pos: (i32, i32),
        size: (i32, i32),
        cursor_name: &str,
        parent_surface: &WlSurface,
        globals: GlobalsHandle<XdgGlobals>,
    ) -> Self {
        let wl_surface = globals.wl_compositor.create_surface();
        let wl_subsurface = globals
            .wl_subcompositor
            .get_subsurface(&wl_surface.detach(), parent_surface);
        wl_subsurface.place_below(&parent_surface);
        wl_subsurface.set_position(pos.0, pos.1);
        let pointer_info = PointerInfo::new(
            cursor_name,
            wl_surface.detach().into(),
            globals.clone(),
        );

        let mut this = Self {
            pos,
            size,
            wl_subsurface,
            shm_pool: globals.shm_pool.clone(),
            pointer_info,
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
        let (buf, wl_buf) = shm_pool
            .buffer(width, height, width * 4, wl_shm::Format::Xrgb8888)
            .expect("Failed to allocate memory");
        buf.fill(0xff);
        self.size = (width, height);
        self.wl_surface.damage_buffer(0, 0, width, height);
        self.wl_surface.attach(Some(&wl_buf), 0, 0);
        self.wl_surface.offset(0, 0);
        self.wl_surface.commit();
    }
}

