
mod globals;
mod window;
mod cursor;
mod frame;

pub use window::XdgWindow;
pub use globals::XdgGlobals;

pub type GlobalsHandle = super::GlobalsHandle<XdgGlobals>;
