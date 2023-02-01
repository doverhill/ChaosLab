use crate::{ServiceObserver};

#[derive(PartialEq)]
pub struct Service<'a, SO: ServiceObserver> {
    // pub on_connected: Option<Box<dyn Fn(ServiceHandle, ChannelHandle) + 'a>>,
    pub observers: Vec<&'a SO>,
}

impl<'a, SO: ServiceObserver + PartialEq> Service<'a, SO> {
    pub fn new() -> Self {
        Service {
            observers: Vec::new(),
        }
    }

    pub fn attach_observer(&mut self, observer: &'a SO) {
        self.observers.push(observer);
    }

    pub fn detach_observer(&mut self, observer: &'a SO) {
        if let Some(index) = self.observers.iter().position(|x| *x == observer) {
            self.observers.remove(index);
        }
    }
}
