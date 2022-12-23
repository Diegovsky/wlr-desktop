
pub struct Nullable<T>(Option<T>);

impl<T> Nullable<T> {
    pub const fn null() -> Self {
        Self(None)
    } 
}

impl<T> Nullable<T> {
    pub fn put(&mut self, val: T) {
        self.0 = Some(val);
    }
}

impl<T> std::default::Default for Nullable<T> {
    fn default() -> Self {
        Self(None)    
    }
}

impl<T> std::ops::Deref for Nullable<T> {
    type Target = T;
    #[track_caller]
    fn deref(&self) -> &Self::Target {
        self.0.as_ref().unwrap()
    }
}

impl<T> std::ops::DerefMut for Nullable<T> {
    #[track_caller]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut().unwrap()
    }
}
