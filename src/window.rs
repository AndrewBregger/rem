
type WResult<T> = Result<T, glfw::Error>;

pub struct Window {
    window: glfw::Window,
}


impl Window {
    pub fn new() -> WResult<(Self, Event)> {
         
    }
}
