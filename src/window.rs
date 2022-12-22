use std::{rc::Rc, collections::HashMap};

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
    cache: RcCell<HashMap<(i32, i32), WlBuffer>>,
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
            cache: Default::default(),
            buffer: None,
        }
    }
    pub fn resize(&mut self, width: i32, height: i32) {
        let mut cache = self.cache.borrow_mut();
        let wlbuf = cache.entry((width, height)).or_insert_with({ let cache = self.cache.clone();
            move || {
            let mut shm_pool = self.shm_pool.borrow_mut();
            let (buf, wlbuf) = shm_pool
                .buffer(
                    width,
                    height,
                    width * 4,
                    wayland_client::protocol::wl_shm::Format::Xrgb8888,
                    )
                .unwrap();
            let _ = self.buffer.insert(wlbuf.clone());
            for (i, pixels) in buf.chunks_exact_mut(4).enumerate() {
                pixels[1] = ((i % width as usize)*255 / width as usize) as u8;
                pixels[2] = ((i / width as usize)*255 / height as usize) as u8;
                pixels[3] = 127;
            }
            wlbuf.quick_assign(move |_wlbuf, evt, _| {
                cache.borrow_mut().remove(&(width, height));
            });
            wlbuf
        } });


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
    fn window_common(&self) -> WindowCommon;
}
