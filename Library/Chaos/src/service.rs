pub struct Service {
    // pub on_connected: Option<Box<dyn Fn(ServiceHandle, ChannelHandle) + 'a>>,
    // pub observers: Vec<&'a mut SO>,
}

impl Service {
    pub fn new() -> Self {
        Service {
        }
    }

    // pub fn attach_observer(&mut self, observer: &'a mut SO) {
    //     self.observers.push(observer);
    // }

    // pub fn detach_observer(&mut self, observer: &'a mut SO) {
    //     if let Some(index) = self.observers.iter().position(|x| *x == observer) {
    //         self.observers.remove(index);
    //     }
    // }
}
