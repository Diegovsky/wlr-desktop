use super::{GlobalsHandle, globals};
use crate::get_globals;
use crate::prelude::RcCell;
use wayland_client::protocol::wl_pointer::{self, WlPointer};
use wayland_client::protocol::wl_surface::WlSurface;
use wayland_client::Main;


#[derive(Debug)]
pub struct CursorFrame {
    surface: Main<WlSurface>,
    hotspot: (u32, u32),
}

impl CursorFrame {
    pub fn new(globals: &GlobalsHandle, name: &str) -> Option<Self> {
        let mut cursor_theme = globals.backend.cursor_theme.borrow_mut();
        let cursor_image = &cursor_theme.get_cursor(name)?[0];
        let cursor_surface = globals.wl_compositor.create_surface();
        let (width, height) = cursor_image.dimensions();
        cursor_surface.attach(Some(&cursor_image), 0, 0);
        cursor_surface.damage_buffer(0, 0, width as i32, height as i32);
        cursor_surface.commit();
        Some(Self { surface: cursor_surface, hotspot: cursor_image.hotspot() })
    }
    pub fn set(&self, serial: u32, ptr: &WlPointer) {
        let (hx, hy) = self.hotspot;
        ptr.set_cursor(serial, Some(&self.surface), hx as i32, hy as i32);
    }
}

trait ClickedCb = Fn(i32, i32);

struct PointerInfoInner {
    wl_surface: WlSurface,
    cursor_frame: CursorFrame,
    cursor_pos: (f64, f64),
    serial: u32,
    is_inside: bool,
    clicked: Option<Box<dyn ClickedCb>>,
}

impl std::fmt::Debug for PointerInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let inner = self.inner.borrow();
        f.debug_struct(std::any::type_name::<Self>())
            .field("wl_surface", &inner.wl_surface)
            .field("cursor_frame", &inner.cursor_frame)
            .field("cursor_pos", &inner.cursor_pos)
            .field("serial", &inner.serial)
            .field("is_inside", &inner.is_inside)
            .field("clicked", &"Cool closure")
            .finish()
    }
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
                    self.cursor_frame.set(self.serial, &ptr)
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
            wl_pointer::Event::Button { serial, time, button, state } => {
                self.serial = serial;
            }
            wl_pointer::Event::Leave { serial, surface } => {
                if self.is_inside && surface == self.wl_surface {
                    self.is_inside = false;
                    self.serial = serial;
                    ptr.set_cursor(self.serial, None, 0, 0)
                }
            }
            _ => (),
        }
    }
    fn update_pointer_pos(&mut self, x: f64, y: f64) {
        self.cursor_pos = (x, y);
    }
}

#[derive(Clone)]
pub struct PointerInfo {
    inner: RcCell<PointerInfoInner>,
}

impl PointerInfo {
    pub fn new(cursor_name: &str, parent_surface: WlSurface) -> Self {
        let globals = get_globals();
        let Some(cursor_frame) = CursorFrame::new(&globals, cursor_name) else {
            panic!("Cursor surface {} not found", cursor_name);
        };
        let this = Self {
            inner: PointerInfoInner {
                wl_surface: parent_surface,
                cursor_frame,
                cursor_pos: (0.0, 0.0),
                serial: 0,
                is_inside: false,
                clicked: None,
            }
            .into(),
        };
        globals.wl_seat.get_pointer().quick_assign({
            let this = this.clone();
            move |ptr, evt, _| this.inner.borrow_mut().handle_pointer_event(evt, ptr)
        });
        this
    }
    pub fn on_clicked(self, on_click: impl ClickedCb + 'static) -> Self {
         self.inner.borrow_mut().clicked = Some(Box::new(on_click));
         self
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
    pub fn update_pointer_pos(&self, x: f64, y: f64) {
        self.inner.borrow_mut().update_pointer_pos(x, y)
    }
}
