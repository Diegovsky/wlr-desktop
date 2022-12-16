use std::rc::Rc;

use rgb::alt::ARGB8;
use rgb::AsPixels;
use smithay_client_toolkit::shm::AutoMemPool;
use wayland_client::{
    protocol::{wl_buffer::WlBuffer, wl_surface::WlSurface},
    Main,
};
use wayland_cursor::CursorImageBuffer;

use crate::{globals::BackendGlobals, prelude::*};

pub mod xdg_shell;

// Assumed to br XRGB
pub struct Pixels<'a> {
    pub buf: &'a [ARGB8],
    pub width: usize,
}

#[derive(Debug, Clone)]
pub struct WindowCommon {
    shm_pool: RcCell<AutoMemPool>,
    surface: Main<WlSurface>,
    buffer: Option<WlBuffer>,
    should_close: bool,
    width: i32,
    height: i32,
}

impl WindowCommon {
    pub fn new(globals: GlobalsHandle<impl BackendGlobals>) -> Self {
        let surface = globals.wl_compositor.create_surface();

        Self {
            shm_pool: globals.shm_pool.clone(),
            surface,
            width: 0,
            height: 0,
            should_close: false,
            buffer: None,
        }
    }
    pub fn resize(&mut self, width: i32, height: i32) {
        let mut shm_pool = self.shm_pool.borrow_mut();
        let (buf, wlbuf) = shm_pool
            .buffer(
                width,
                height,
                width * 4,
                wayland_client::protocol::wl_shm::Format::Xrgb8888,
            )
            .unwrap();
        let wlbuf = self.buffer.insert(wlbuf);
        for (i, pixels) in buf.chunks_exact_mut(4).enumerate() {
            pixels[1] = ((i % width as usize)*255 / width as usize) as u8;
            pixels[2] = ((i / width as usize)*255 / height as usize) as u8;
            pixels[3] = 127;
        }

        let surface = self.surface.detach();
        surface.attach(Some(wlbuf), 0, 0);
        surface.damage_buffer(0, 0, width, height);

        self.width = width;
        self.height = height;

        surface.commit();
    }
}

pub trait WindowBackend: Clone {
    type BackendGlobals: BackendGlobals<Window = Self>;
    fn new(globals: GlobalsHandle<Self::BackendGlobals>) -> Self;
    fn should_close(&self) -> bool {
        self.window_common().should_close
    }
    fn attach_cursor(&self, cursor_image: &CursorImageBuffer) {
        let win = self.window_common();
        let size = cursor_image.dimensions();
        win.surface.damage_buffer(0, 0, size.0 as i32, size.1 as i32);
        win.surface.attach(Some(&cursor_image), 0, 0);
        win.surface.commit();
    }
    fn draw(&self, buf: &[u8], width: usize) {
        let win = self.window_common();
        let mut shm_pool = win.shm_pool.borrow_mut();
        let size = (width as i32, (buf.len()/4/width) as i32);
        let (new_buf, wl_buf) = shm_pool.buffer(size.0, size.1, size.0*4, wayland_client::protocol::wl_shm::Format::Xrgb8888).expect("Failed to allocate memory");
        new_buf.copy_from_slice(buf);
        win.surface.damage_buffer(0, 0, size.0, size.1);
        win.surface.attach(Some(&wl_buf), 0, 0);
        win.surface.commit();
    }
    fn window_common(&self) -> WindowCommon;
}
