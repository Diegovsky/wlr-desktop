use std::rc::Rc;

use smithay_client_toolkit::shm::AutoMemPool;
use wayland_client::{
    protocol::{
        wl_compositor::WlCompositor, wl_shm::WlShm, wl_seat::WlSeat, wl_subcompositor::WlSubcompositor, wl_pointer::WlPointer,
    }, GlobalError, GlobalManager, Interface, Main, Proxy, Display,
};

use crate::prelude::*;

use crate::window::WindowBackend;


pub struct GlobalsInner<B: BackendGlobals> {
    pub global_manager: GlobalManager,
    pub wl_compositor: Main<WlCompositor>,
    pub wl_seat: Main<WlSeat>,
    pub wl_shm: Main<WlShm>,
    pub wl_subcompositor: Main<WlSubcompositor>,
    pub display: Display,
    pub wl_pointer: Main<WlPointer>,

    pub shm_pool: RcCell<AutoMemPool>,
    pub backend: Rc<B>,
}


impl<B: BackendGlobals> GlobalsHandle<B> {
    pub fn new(global_manager: GlobalManager, display: &Display) -> Self {
        let shm = global_manager.get::<WlShm>();
        let wl_seat: Main<WlSeat> = global_manager.get();
        let inner = GlobalsInner {
            display: display.clone(),
            wl_compositor: global_manager.get(),
            wl_pointer: wl_seat.get_pointer(),
            wl_seat,
            wl_subcompositor: global_manager.get(),
            wl_shm: shm.clone(),
            backend: B::new(&global_manager),
            shm_pool: RcCell::new(AutoMemPool::new(shm.into()).unwrap()),
            global_manager,
        };
        let inner = Rc::new(inner);
        Self { inner }
    }
    pub fn new_window(&self) -> B::Window {
        B::Window::new(self.clone())
    }
}

pub trait BackendGlobals {
    type Window: WindowBackend<BackendGlobals=Self>;
    fn new(global_manager: &GlobalManager) -> Rc<Self>;
}

pub trait GlobalManagerExt {
    fn instantiate_current<I>(&self) -> Result<Main<I>, GlobalError>
    where
        I: Interface + AsRef<Proxy<I>> + From<Proxy<I>>;

    fn get<I>(&self) -> Main<I>
    where
        I: Interface + AsRef<Proxy<I>> + From<Proxy<I>>,
    {
        self.instantiate_current().unwrap()
    }
}
impl GlobalManagerExt for GlobalManager {
    fn instantiate_current<I>(&self) -> Result<Main<I>, GlobalError>
    where
        I: Interface + AsRef<Proxy<I>> + From<Proxy<I>>,
    {
        self.instantiate_exact(I::VERSION)
    }

    fn get<I>(&self) -> Main<I>
    where
        I: Interface + AsRef<Proxy<I>> + From<Proxy<I>>,
    {
        match self.instantiate_current() {
            Ok(val) => val,
            Err(e) => {
                panic!("Error trying to bind interface {}. {:?}", I::NAME, e)
            },
        }
    }
}

pub struct GlobalsHandle<B: BackendGlobals> {
    inner: Rc<GlobalsInner<B>>,
}

impl<B: BackendGlobals> Clone for GlobalsHandle<B> {
    fn clone(&self) -> Self {
        Self { inner: self.inner.clone() }
    }
}

impl<B: BackendGlobals> std::ops::Deref for GlobalsHandle<B> {
    type Target = GlobalsInner<B>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
