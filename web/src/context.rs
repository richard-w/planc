use std::{
    cell::RefCell,
    ops::{Deref, DerefMut},
};

#[derive(Debug, Default, PartialEq)]
pub struct AppContext {
    name: RefCell<Option<String>>,
}

impl AppContext {
    /// Name of the participant
    pub fn name<'a>(&'a self) -> impl Deref<Target = Option<String>> + 'a {
        self.name.borrow()
    }

    /// Name of the participant (mutable)
    pub fn name_mut<'a>(&'a self) -> impl Deref<Target = Option<String>> + DerefMut + 'a {
        self.name.borrow_mut()
    }
}
