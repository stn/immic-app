pub mod application;
pub mod filelog;
pub mod screen;

pub trait Plugin {
    fn start(&mut self);
    fn stop(&mut self);
}
