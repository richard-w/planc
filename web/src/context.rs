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
    pub fn name(&self) -> impl Deref<Target = Option<String>> + '_ {
        self.name.borrow()
    }

    /// Name of the participant (mutable)
    pub fn name_mut(&self) -> impl Deref<Target = Option<String>> + DerefMut + '_ {
        self.name.borrow_mut()
    }
}
