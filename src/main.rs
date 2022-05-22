use std::collections::{HashMap, hash_map::{Entry, Keys}};
use std::thread::sleep;
use std::time::Duration;
use std::ops::{Add, AddAssign, Sub};
use std::hash::Hash;
use pancurses::*;

const INIT: &'static [&str] = &[
    // "X X",
    // " XX",
    // " X ",

    "                        X           ",
    "                      X X           ",
    "            XX      XX            XX",
    "           X   X    XX            XX",
    "XX        X     X   XX              ",
    "XX        X   X XX    X X           ",
    "          X     X       X           ",
    "           X   X                    ",
    "            XX                      ",
];

type BaseType = i64;

#[derive(Copy, Clone)]
struct Coord<T: Copy> (T, T);

impl<T: Add<Output = T> + Copy> Coord<T> {
    pub fn offset(&self, off: Coord<T>) -> Coord<T> {
        Coord(self.0 + off.0, self.1 + off.1)
    }
}

#[derive(Copy, Clone)]
enum State {
    Alive,
    Dying
}

struct Map<T> {
    map: HashMap<T, HashMap<T, State>>,
}

impl<T: Eq + Hash + Copy + From<i32> + Add<Output = T> + AddAssign> Map<T> {
    pub fn new() -> Map<T> {
        Map {
            map: HashMap::new(),
        }
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

struct MapIter<'a, T> {
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

struct Viewport<'a, T: Copy> {
    win: &'a pancurses::Window,
    origin: Coord<T>,
    size: Coord<T>
}

impl<T> Viewport<'_, T> where
    i32: TryFrom<T>,
    T: From<i32>,
    T: Add<Output = T>,
    T: Sub<Output = T>,
    T: PartialOrd,
    T: Copy,
    T: AddAssign
{
    pub fn new(win: &pancurses::Window) -> Viewport<T> {
        let mx = win.get_max_x();
        let my = win.get_max_y();
        Viewport {
            win,
            origin: Coord((-mx / 2).into(), (-my / 2).into()),
            size: Coord(mx.into(), my.into())
        }
    }

    pub fn is_inside(&self, x: T, y: T) -> bool {
        x >= self.origin.0 &&
        x <= self.origin.0 + self.size.0 &&
        y >= self.origin.1 &&
        y <= self.origin.1 + self.size.1
    }

    pub fn render(&self, map: &Map<T>) {
        self.win.erase();
        for (x, ym) in map.map.iter() {
            for (y, _) in ym.iter() {
                if self.is_inside(*x, *y) {
                    let vpx = i32::try_from(*x - self.origin.0).ok().unwrap();
                    let vpy = i32::try_from(*y - self.origin.1).ok().unwrap();
                    self.win.mvaddch(vpy, vpx, 'O');
                }
            }
        }
        self.win.refresh();
    }

    pub fn mv(&mut self, x: T, y: T) {
        self.origin.0 += x;
        self.origin.1 += y;
    }
}

fn main() {
    let slp = Duration::from_millis(10);

    let mut map: Map<BaseType> = Map::new_from_str_array(INIT);

    let win = initscr();
    curs_set(0);
    win.nodelay(true);
    win.keypad(true);

    let mut viewport: Viewport<BaseType> = Viewport::new(&win);

    loop {
        viewport.render(&map);

        {
            let mut dying: Vec<Coord<BaseType>> = Vec::new();

            for i in map.iter() {
                let nc = map.ncount(i);

                if nc < 2 || nc > 3 {
                    dying.push(i);
                }
            }

            for d in dying {
                map.set(d, State::Dying);
            }
        }

        {
            let mut alive: Vec<Coord<BaseType>> = Vec::new();

            for i in map.iter() {
                for dx in -1..2 {
                    for dy in -1..2 {
                        let c = Coord(i.0 + dx, i.1 + dy);
                        let nc = map.ncount(c.clone());
                        if nc == 3 {
                            alive.push(c);
                        }
                    }
                }
            }

            for a in alive {
                map.set(a, State::Alive);
            }
        }

        {
            let mut kill: Vec<Coord<BaseType>> = Vec::new();

            for i in map.iter() {
                if let Some(s) = map.get(i) {
                    match s {
                        State::Dying => kill.push(i),
                        State::Alive => (),
                    }
                }
            }

            for k in kill {
                map.kill(k);
            }
        }

        map.gc();

        if let Some(ch) = win.getch() {
            match ch {
                Input::KeyLeft => viewport.mv(-1, 0),
                Input::KeyUp => viewport.mv(0, -1),
                Input::KeyRight => viewport.mv(1, 0),
                Input::KeyDown => viewport.mv(0, 1),
                Input::Character(c) => {
                    if c == 'q' {
                        break;
                    }
                },
                _ => ()
            }
        }

        sleep(slp);
    }

    endwin();
}
