
pub struct Nullable<T>(Option<T>);

impl<T> Nullable<T> {
    pub fn put(&mut self, val: T) {
        self.0.insert(val);
    }
}

impl<T> std::default::Default for Nullable<T> {
    fn default() -> Self {
        return Self(None)    
    }
}

impl<T> std::ops::Deref for Nullable<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.0.as_ref().unwrap()
    }
}

impl<T> std::ops::DerefMut for Nullable<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut().unwrap()
    }
}
