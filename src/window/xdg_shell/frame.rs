use smithay_client_toolkit::shm::AutoMemPool;
use wayland_client::protocol::wl_shm;
use wayland_client::protocol::wl_subsurface::WlSubsurface;
use wayland_client::protocol::wl_surface::WlSurface;
use wayland_client::Main;

use crate::prelude::*;

use super::GlobalsHandle;
use super::cursor::PointerInfo;

#[derive(Debug)]
pub struct XdgWindowFrame {
    padding: i32,
    // laterals: [XdgWindowBorder; 4],
    corners: [XdgWindowBorder; 4],
}

impl XdgWindowFrame {
    pub fn new(parent: &WlSurface, padding: u32, globals: GlobalsHandle) -> Self {
        let padding = padding as i32;
        let make_border = |cursor_name, dir| {
            XdgWindowBorder::new(
                (0, 0),
                (padding, padding),
                cursor_name,
                parent,
                dir,
                globals.clone(),
            )
        };
        let names_dirs = [
            ("top_left_corner", DiagOrientation::TopLeft),
            ("top_right_corner", DiagOrientation::TopRight),
            ("bottom_left_corner", DiagOrientation::BottomLeft),
            ("bottom_right_corner", DiagOrientation::BottomRight),
        ];

        // let laterals = todo!();
        let corners = names_dirs.map(|(name, dir)| make_border(name, dir));
        Self { corners, padding }
    }
    pub fn resize(&mut self, width: i32, height: i32) {

    }
    pub fn move_(&mut self, width: i32, height: i32) {
        /* let corners_pos = [
            [0, 0], // top-left
            [1, 0], // top-right
            [0, 1], // bottom-left
            [1, 1], // bottom-right
        ]; */
        let parent_size = [width, height];
        for (i, corner) in self.corners.iter_mut().enumerate() {
            let corner = corner.dir.get_pos(id, c);
            let x = corners_pos[i][0] * parent_size[0] - (self.padding / 2);
            let y = corners_pos[i][1] * parent_size[1] - (self.padding / 2);
            corner.resize(self.padding, self.padding);
            corner.move_(x, y);
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum DiagOrientation {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl DiagOrientation {
    fn vector(&self) -> [i32; 2] {
        match *self {
            Self::TopLeft=>    [0, 0], // top-left
            Self::TopRight=>   [1, 0], // top-right
            Self::BottomLeft=> [0, 1], // bottom-left
            Self::BottomRight=>[1, 1], // bottom-right
        }
    }
}

#[derive(Debug)]
enum Orientation {
    Top,
    Left,
    Right,
    Bottom
}

impl Orientation {
    fn vector(&self) -> [f32; 2] {
        match self {
            Orientation::Top => [0.5, 0.0],
            Orientation::Left => [0.0, 0.5],
            Orientation::Right => [1.0, 0.5],
            Orientation::Bottom => [0.5, 1.0],
        }
    }
}

#[derive(Debug)]
enum Dir {
    Diagonal(DiagOrientation),
    Cardinal(Orientation)
}

impl Dir {
   fn translate_coords(&self, coords: [i32; 2]) -> [i32; 2] {
       match self {
           Dir::Diagonal(diag) => diag.vector().zip_map(coords, |(displacement, c)| displacement*c),
           Dir::Cardinal(card) => (card.vector()[id] * c as f32) as i32
       }
   }
}

impl std::convert::From<Orientation> for Dir {
 fn from(value: Orientation) -> Self {
     Self::Cardinal(value)
 }
}

impl std::convert::From<DiagOrientation> for Dir {
 fn from(value: DiagOrientation) -> Self {
     Self::Diagonal(value)
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
    dir: Dir,
}

impl XdgWindowBorder {
    fn new(
        pos: (i32, i32),
        size: (i32, i32),
        cursor_name: &str,
        parent_surface: &WlSurface,
        dir: Dir,
        globals: GlobalsHandle,
    ) -> Self {
        let wl_surface = globals.wl_compositor.create_surface();
        let wl_subsurface = globals
            .wl_subcompositor
            .get_subsurface(&wl_surface.detach(), parent_surface);
        wl_subsurface.place_below(&parent_surface);
        wl_subsurface.set_position(pos.0, pos.1);
        let pointer_info = PointerInfo::new(
            cursor_name,
            wl_surface.detach(),
        ).on_clicked(|x, y| {
            println!("{},{}", x, y)
        });

        let mut this = Self {
            pos,
            size,
            wl_subsurface,
            shm_pool: globals.shm_pool.clone(),
            pointer_info,
            dir,
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

