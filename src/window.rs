use std::{rc::Rc, collections::HashMap};

use rgb::alt::ARGB8;
use rgb::AsPixels;
use smithay_client_toolkit::shm::AutoMemPool;
use wayland_client::{
    protocol::{wl_buffer::WlBuffer, wl_surface::WlSurface},
    Main, QueueToken,
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
        for (i, pixels) in buf.chunks_exact_mut(4).enumerate() {
            let r = (i % width as usize) * 255 / width as usize;
            let g = i*255 / (width * height) as usize;
            let bor = 255usize;

            let mut b = bor.overflowing_sub(r);
            if b.1 {
                b = r.overflowing_sub(bor);
            }
            pixels[0] = b.0 as u8;
            pixels[1] = g as u8;
            pixels[2] = r as u8;
        }

        self.surface.attach(Some(&wlbuf), 0, 0);
        self.surface.damage_buffer(0, 0, width, height);

        self.width = width;
        self.height = height;

        self.surface.commit();
    }
}

pub trait WindowBackend: Clone {
    type BackendGlobals: BackendGlobals<Window = Self>;
    fn new(globals: GlobalsHandle<Self::BackendGlobals>) -> Self;
    fn should_close(&self) -> bool {
        self.window_common().should_close
    }
    fn window_common(&self) -> WindowCommon;
}
