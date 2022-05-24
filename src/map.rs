use std::collections::hash_map::{Entry, Keys};
use std::collections::HashMap;

use crate::coord::Coord;
use crate::state::State;

use std::hash::Hash;
use std::ops::Add;

use std::ops::AddAssign;

pub struct Map<T> {
    map: HashMap<T, HashMap<T, State>>,
}

impl<T: Eq + Hash + Copy + From<i32> + Add<Output = T> + AddAssign> Map<T> {
    pub fn new() -> Map<T> {
        Map {
            map: HashMap::new(),
        }
    }

    pub fn map(&self) -> &HashMap<T, HashMap<T, State>> {
        &self.map
    }

    pub fn set(&mut self, coord: Coord<T>, s: State) {
        let ymap = self.map.entry(coord.0).or_insert(HashMap::new());
        ymap.insert(coord.1, s);
    }

    pub fn get(&self, coord: Coord<T>) -> Option<State> {
        if let Some(xv) = self.map.get(&coord.0) {
            if let Some(yv) = xv.get(&coord.1) {
                return Some(*yv);
            }
        }
        None
    }

    pub fn kill(&mut self, coord: Coord<T>) {
        if let Entry::Occupied(mut xo) = self.map.entry(coord.0) {
            if let Entry::Occupied(yo) = xo.get_mut().entry(coord.1) {
                yo.remove();
            }
        }
    }

    pub fn gc(&mut self) {
        let mut unused: Vec<T> = Vec::new();
        for (x, ym) in self.map.iter_mut() {
            if ym.is_empty() {
                unused.push(*x);
            }
        }

        for x in &unused {
            self.map.remove(x);
        }
    }

    pub fn ncount(&self, coord: Coord<T>) -> u8 {
        let mut n = 0;
        for dx in -1..2 {
            for dy in -1..2 {
                if !(dx == 0 && dy == 0) {
                    if let Some(_st) = self.get(coord.offset(Coord(dx.into(), dy.into()))) {
                        n += 1;
                    }
                }
            }
        }
        n
    }

    pub fn new_from_str_array<'a>(s: &[&str]) -> Map<T> {
        let mut newmap: Map<T> = Map::new();
        let mut y: T = 0.into();
        let mut x: T;
        for row in s {
            x = 0.into();
            for ch in row.chars() {
                if ch != ' ' {
                    newmap.set(Coord(x, y), State::Alive);
                }
                x += 1.into();
            }
            y += 1.into();
        }

        newmap
    }

    pub fn iter(&self) -> MapIter<T> {
        MapIter {
            map: self,
            xkeys: None,
            xkey: None,
            ykeys: None,
            ykey: None,
        }
    }
}

pub struct MapIter<'a, T> {
    map: &'a Map<T>,
    xkeys: Option<Keys<'a, T, HashMap<T, State>>>,
    xkey: Option<&'a T>,
    ykeys: Option<Keys<'a, T, State>>,
    ykey: Option<&'a T>,
}

impl<'a, T: Eq + Hash> MapIter<'_, T> {
    fn next_xkey(&mut self) -> bool {
        match &mut self.xkeys {
            Some(keys) => {
                self.xkey = keys.next();
                if self.xkey.is_some() {
                    if let Some(yval) = self.map.map.get(self.xkey.unwrap()) {
                        self.ykeys = Some(yval.keys());
                        self.ykey = None;
                    } else {
                        panic!("Empty X hashmap");
                    }
                    true
                } else {
                    self.ykeys = None;
                    self.ykey = None;
                    false
                }
            },
            None => false,
        }
    }

    fn next_ykey(&mut self) -> bool {
        match &mut self.ykeys {
            Some(keys) => {
                self.ykey = keys.next();
                self.ykey.is_some()
            },
            None => false,
        }
    }
}

impl<'a, T: Copy + Eq + Hash> Iterator for MapIter<'a, T> {
    type Item = Coord<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.xkeys.is_none() {
            self.xkeys = Some(self.map.map.keys());
            if !self.next_xkey() {
                return None;
            }
        }

        while !self.next_ykey() {
            if !self.next_xkey() {
                return None;
            }
        }

        Some(Coord(*self.xkey.unwrap(), *self.ykey.unwrap()))
    }
}
