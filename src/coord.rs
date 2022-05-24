use std::ops::Add;

#[derive(Copy, Clone)]
pub struct Coord<T: Copy> (pub T, pub T);

impl<T: Add<Output = T> + Copy> Coord<T> {
    pub fn offset(&self, off: Coord<T>) -> Coord<T> {
        Coord(self.0 + off.0, self.1 + off.1)
    }
}
