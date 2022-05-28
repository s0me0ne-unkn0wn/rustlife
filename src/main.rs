// use std::collections::{HashMap, hash_map::{Entry, Keys}};
// use std::thread::sleep;
use std::time::{Duration, SystemTime};
use std::ops::{Add, AddAssign, Sub};
use std::hash::Hash;
use std::io::{self, BufReader, BufRead};
use std::path::{Path};
use pancurses::*;

pub mod coord;
pub mod state;
pub mod map;

use coord::Coord;
use state::State;
use map::Map;

const INIT: &[&str] = &[
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

fn read_lines<P: AsRef<Path>>(file: P) -> io::Result<Vec<String>> {
    BufReader::new(std::fs::File::open(file)?).lines().collect()
}

// TODO: Refactor: separate parsers to a module (or integrate with Map)
fn read_rle<P: AsRef<Path>>(file: P) -> io::Result<Vec<String>> {
    let rle = read_lines(file)?;
    let mut res: Vec<String> = Vec::new();
    let mut numstr = String::new();
    let mut str = String::new();
    for l in rle {
        if l.starts_with("#") {
            // println!("Ignored comment {}", l);
        } else if l.starts_with("x") {
            // println!("Ignored rule set {}", l);
        } else {
            // println!("Parsing {}", l);
            for c in l.chars() {
                match c {
                    '0' ..= '9' => {
                        numstr += &c.to_string();
                    },
                    'b' | 'o' => {
                        let num;
                        if let Ok(p) = numstr.parse::<i32>() {
                            num = p;
                        } else {
                            num = 1;
                        }
                        for _ in 0..num {
                            str += &(if c == 'b' { ' ' } else { 'X' }).to_string();
                        }
                        numstr = String::new();
                    },
                    '$' => {
                        res.push(str);
                        str = String::new();
                    },
                    '!' => {
                        if !str.is_empty() {
                            res.push(str);
                        }
                        return Ok(res);
                    }
                    _ => ()
                }
            }
        }
    }
    Err(io::Error::new(io::ErrorKind::InvalidData, "Cannot parse RLE file"))
}

fn main() {

    let mut map: Map<BaseType> = Map::new_from_str_array(INIT.to_vec());

    let win = initscr();
    curs_set(0);
    win.nodelay(true);
    win.keypad(true);

    let mut viewport: Viewport<BaseType> = Viewport::new(&win);

    let mut turn = 0u64;
    let mut cells = 0u64;

    let mut delay = Duration::from_millis(128);
    let mut do_delay = true;
    let mut last_now = SystemTime::now();
    let mut running = true;

    loop {
        let now = SystemTime::now();

        if running && (!do_delay || now.duration_since(last_now).unwrap_or(Duration::from_millis(0)) > delay) {
            turn += 1;
            last_now = now;

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
        } else {
            viewport.render(&map);
        }

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
                        if delay.as_millis() > 1 {
                            delay /= 2;
                        }
                    } else if c == 'o' {
                        running = false;
                        let fowin = win.subwin(
                            win.get_max_y() / 2,
                            win.get_max_x() / 2,
                            win.get_max_y() / 4,
                            win.get_max_x() / 4,
                        ).expect("Cannot create subwindow");

                        fowin.keypad(true);

                        let line_width: usize = (win.get_max_x() / 2 - 4).try_into().unwrap();
                        let num_lines: usize = (win.get_max_y() / 2 - 2).try_into().unwrap();

                        // TODO: Refactor: separate new window and menu logic from FS
                        // Menu window should be a module, and FS logic a function
                        'dir: loop {
                            let mut cursor = 1;
                            let mut first_line: usize = 0;
                            let cwd = std::env::current_dir().unwrap();
                            fowin.erase();
                            let mut entries = std::fs::read_dir(&cwd).unwrap().map(|res| {
                                let path = res.unwrap().path();
                                (path.file_name().unwrap().to_string_lossy().into_owned(), path)
                            }).collect::<Vec<_>>();
                            entries.sort_by(|a, b| {
                                if a.1.is_dir() && !b.1.is_dir() {
                                    return std::cmp::Ordering::Less;
                                }
                                if !a.1.is_dir() && b.1.is_dir() {
                                    return std::cmp::Ordering::Greater;
                                }
                                a.cmp(b)
                            });
                            let mut up = cwd.clone();
                            up.push(std::path::Path::new(".."));
                            entries.insert(0, (String::from(".."), up));
                            loop {
                                fowin.attroff(A_REVERSE);
                                fowin.attron(A_ALTCHARSET);
                                fowin.border(ACS_VLINE(), ACS_VLINE(), ACS_HLINE(), ACS_HLINE(), ACS_ULCORNER(), ACS_URCORNER(), ACS_LLCORNER(), ACS_LRCORNER());
                                fowin.attroff(A_ALTCHARSET);
                                fowin.mv(0, 4);
                                fowin.addstr(format!(" {} ", cwd.to_string_lossy()));
                                let mut line = 1;
                                for e in &entries[first_line..(std::cmp::min(first_line + num_lines, entries.len()))] {
                                    // let canonical = e.canonicalize().unwrap();
                                    // let mut fname = canonical.file_name().unwrap().to_string_lossy();
                                    let mut fname = e.0.clone();
                                    if e.1.is_dir() {
                                        fname += "/";
                                    }
                                    if line == cursor {
                                        fowin.attron(A_REVERSE);
                                    } else {
                                        fowin.attroff(A_REVERSE);
                                    }
                                    fowin.mvaddstr(line.try_into().unwrap(), 2, format!("{:<1$}", fname, line_width));
                                    line += 1;
                                }

                                fowin.refresh();

                                if let Some(ch) = fowin.getch() {
                                    match ch {
                                        Input::KeyDown => {
                                            if cursor == line - 1 && first_line + num_lines < entries.len() {
                                                first_line += 1;
                                                fowin.erase();
                                            }
                                            if cursor < line - 1 {
                                                cursor += 1;
                                            }
                                        },
                                        Input::KeyUp => {
                                            if cursor == 1 && first_line > 0 {
                                                first_line -= 1;
                                                fowin.erase();
                                            }
                                            if cursor > 1 {
                                                cursor -= 1;
                                            }
                                        },
                                        Input::Character(c) => {
                                            if c == '\x0a' {
                                                let e = &entries[first_line + cursor - 1];
                                                if e.1.is_dir() {
                                                    let mut newcwd = cwd.clone();
                                                    newcwd.push(&e.1);
                                                    std::env::set_current_dir(newcwd);
                                                    continue 'dir;
                                                } else if e.0.to_lowercase().ends_with(".rle") {
                                                    let arr = read_rle(&e.1).unwrap();
                                                    map = Map::new_from_str_array(arr);
                                                    running = true;
                                                    break 'dir;
                                                }
                                            } else {
                                                ()
                                            }
                                        }
                                        _ => ()
                                    }
                                }
                            }
                        }
                    }
                },
                _ => ()
            }
        }
    }

    endwin();
}
