use std::cell::{Ref, RefMut};
use std::{rc::Rc, cell::RefCell};

pub use crate::globals::GlobalsHandle;
pub use crate::nullable::Nullable;

#[derive(Debug, Default)]
pub struct RcCell<T: ?Sized>(Rc<RefCell<T>>);

impl<T: ?Sized> Clone for RcCell<T> {
    fn clone(&self) -> Self {
         RcCell(self.0.clone())
     } 
}
pub struct Weak<T: ?Sized>(std::rc::Weak<RefCell<T>>);

pub use crate::globals::GlobalManagerExt;

impl<T: ?Sized> Weak<T> {
   pub fn try_strong(&self) -> Option<RcCell<T>> {
       self.0.upgrade().map(RcCell)
   }
}

impl<T> RcCell<T> {
    pub fn new(value: T) -> Self {
        Self(Rc::new(RefCell::new(value)))
    }
}

impl<T: ?Sized> RcCell<T> {
    pub fn borrow(&self) -> Ref<'_, T> {
        self.bor()
    }
    pub fn borrow_mut(&self) -> RefMut<'_, T> {
        self.bor_mut()
    }
    pub fn bor(&self) -> Ref<'_, T> {
        (*self.0).borrow()
    }
    pub fn bor_mut(&self) -> RefMut<'_, T> {
        (*self.0).borrow_mut()
    }
    pub fn weak(&self) -> Weak<T> {
        Weak(Rc::downgrade(&self.0))
    }
}

impl<T> std::convert::From<T> for RcCell<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

pub fn null<T>() -> Nullable<T> {
    Default::default()
}
