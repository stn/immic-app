pub mod application;
pub mod browser;
pub mod filelog;
pub mod screen;
pub mod setting;

pub trait Plugin {
    fn start(&mut self);
    fn stop(&mut self);
}
