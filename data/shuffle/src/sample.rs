use std::iter::repeat;

use rand::Rng;

#[derive(Hash, Eq, PartialEq)]
pub struct Sample(pub u16, pub u16);

impl Sample{
    pub fn iterate(width: u16, height: u16) -> impl Iterator<Item = Sample>{
        let width_iter = (0..width).cycle();
        let height_iter = (0..height).map(move |v|repeat(v).take(width as usize)).flatten();
        let iter = width_iter.zip(height_iter);
        iter.map(|(a, b)|Sample(a, b))
    }
}

pub struct Reservoir<T, R: Rng>{
    rand: R,
    count: u32,
    current: Option<T>
}

impl<T, R: Rng> Reservoir<T, R>{
    pub fn new(rand: R) -> Self{
        Self{
            count: 0,
            current: None,
            rand,
        }
    }
    
    ///Gets currently chosen element
    pub fn get_current(&self) -> Option<&T>{
        self.current.as_ref()
    }

    ///Gets handle to Reservoir. A return value of `None` means the Reservoir should not be updated.
    pub fn get_handle(&mut self) -> Option<&mut Option<T>>{
        self.count += 1;
        if self.rand.gen_ratio(1, self.count) {
            Some(&mut self.current)
        }else{
            None
        }
    }


}