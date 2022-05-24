// use std::collections::{HashMap, hash_map::{Entry, Keys}};
use std::thread::sleep;
use std::time::{Duration, SystemTime};
use std::ops::{Add, AddAssign, Sub};
use std::hash::Hash;
use pancurses::*;

pub mod coord;
pub mod state;
pub mod map;

use coord::Coord;
use state::State;
use map::Map;

const INIT: &'static [&str] = &[
    // "X X",
    // " XX",
    // " X ",

    // " XX",
    // "XX ",
    // " X ",

    " X     ",
    "   X   ",
    "XX  XXX",

    // "                        X           ",
    // "                      X X           ",
    // "            XX      XX            XX",
    // "           X   X    XX            XX",
    // "XX        X     X   XX              ",
    // "XX        X   X XX    X X           ",
    // "          X     X       X           ",
    // "           X   X                    ",
    // "            XX                      ",
];

type BaseType = i64;


struct Viewport<'a, T: Copy> {
    win: &'a pancurses::Window,
    origin: Coord<T>,
    size: Coord<T>,
    turn: u64, // TODO: Move stats to Map<T>
    cells :u64,
}

impl<T> Viewport<'_, T> where
    i32: TryFrom<T>,
    T: From<i32>,
    T: Add<Output = T>,
    T: Sub<Output = T>,
    T: PartialOrd,
    T: Copy,
    T: AddAssign,
    T: Eq,
    T: Hash,
{
    pub fn new(win: &pancurses::Window) -> Viewport<T> {
        let mx = win.get_max_x();
        let my = win.get_max_y();
        Viewport {
            win,
            origin: Coord((-mx / 2).into(), (-my / 2).into()),
            size: Coord(mx.into(), my.into()),
            turn: 0,
            cells: 0,
        }
    }

    pub fn is_inside(&self, x: T, y: T) -> bool { // TODO: Refactor to accept Coord<T>
        x >= self.origin.0 &&
        x <= self.origin.0 + self.size.0 &&
        y >= self.origin.1 &&
        y <= self.origin.1 + self.size.1
    }

    pub fn render(&self, map: &Map<T>) {
        self.win.erase();
        for (x, ym) in map.map().iter() {
            for (y, _) in ym.iter() {
                if self.is_inside(*x, *y) {
                    let vpx = i32::try_from(*x - self.origin.0).ok().unwrap();
                    let vpy = i32::try_from(*y - self.origin.1).ok().unwrap();
                    self.win.mvaddch(vpy, vpx, 'O');
                }
            }
        }
        self.win.mvaddstr(i32::try_from(self.size.1).ok().unwrap() - 1, 0, format!("Turn: {} Cells: {}", self.turn, self.cells));
        self.win.refresh();
    }

    pub fn mv(&mut self, x: T, y: T) { // TODO: Refactor to accept Coord<T>
        self.origin.0 += x;
        self.origin.1 += y;
    }

    pub fn update_stats(&mut self, turn: u64, cells: u64) {
        self.turn = turn;
        self.cells = cells;
    }
}

fn main() {

    let mut map: Map<BaseType> = Map::new_from_str_array(INIT);

    let win = initscr();
    curs_set(0);
    win.nodelay(true);
    win.keypad(true);

    let mut viewport: Viewport<BaseType> = Viewport::new(&win);

    let mut turn = 0u64;
    let mut cells = 0u64;

    let mut delay = 100;
    let mut do_delay = true;

    loop {
        turn += 1;

        let now = SystemTime::now();

        viewport.update_stats(turn, cells);
        viewport.render(&map);

        cells = 0;

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
                        State::Alive => {
                            cells += 1;
                            ()
                        },
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
                    } else if c == 'd' {
                        do_delay = !do_delay;
                    } else if c == '-' {
                        delay *= 2;
                    } else if c == '+' {
                        if delay > 1 {
                            delay /= 2;
                        }
                    }
                },
                _ => ()
            }
        }

        if do_delay {
            let elapsed = now.elapsed().unwrap();
            let delay_dur = Duration::from_millis(delay);
            if delay_dur > elapsed {
                sleep(delay_dur - elapsed);
            }
        }
    }

    endwin();
}
