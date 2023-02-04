use iter_tools::Itertools;
use log::{debug, error, info, trace, warn};
use std::io;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display)]
pub enum Color {
    Red,
    Green,
    Blue,
}

impl Color {
    pub fn next(self) -> Option<Self> {
        match self {
            Color::Red => Some(Color::Green),
            Color::Green => Some(Color::Blue),
            Color::Blue => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Symbol {
    R,
    G,
    B,
    REnd,
    GEnd,
    BEnd,
    Empty,
    White(u8),
}

impl From<char> for Symbol {
    fn from(c: char) -> Self {
        match c {
            'r' => Symbol::R,
            'g' => Symbol::G,
            'b' => Symbol::B,
            'R' => Symbol::REnd,
            'G' => Symbol::GEnd,
            'B' => Symbol::BEnd,
            '.' => Symbol::Empty,
            '1' => Symbol::White(1),
            '2' => Symbol::White(2),
            '3' => Symbol::White(3),
            '4' => Symbol::White(4),
            _ => panic!("invalid symbol: {}", c),
        }
    }
}

impl Symbol {
    pub fn color(c: Color) -> Self {
        match c {
            Color::Red => Symbol::R,
            Color::Green => Symbol::G,
            Color::Blue => Symbol::B,
        }
    }

    pub fn color_end(c: Color) -> Self {
        match c {
            Color::Red => Symbol::REnd,
            Color::Green => Symbol::GEnd,
            Color::Blue => Symbol::BEnd,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumIter)]
pub enum Direction {
    Right,
    DownRight,
    Down,
    DownLeft,
    Left,
    UpLeft,
    Up,
    UpRight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectionInner {
    Right = 0,
    DownRight = 1,
    Down = 2,
    DownLeft = 3,
}

impl From<u8> for DirectionInner {
    fn from(d: u8) -> Self {
        match d {
            0 => DirectionInner::Right,
            1 => DirectionInner::DownRight,
            2 => DirectionInner::Down,
            3 => DirectionInner::DownLeft,
            _ => panic!("invalid direction: {}", d),
        }
    }
}

impl From<DirectionInner> for Direction {
    fn from(d: DirectionInner) -> Self {
        match d {
            DirectionInner::Right => Direction::Right,
            DirectionInner::DownRight => Direction::DownRight,
            DirectionInner::Down => Direction::Down,
            DirectionInner::DownLeft => Direction::DownLeft,
        }
    }
}

impl Direction {
    pub fn to_inner(self) -> (DirectionInner, bool) {
        match self {
            Direction::Up => (DirectionInner::Down, true),
            Direction::Down => (DirectionInner::Down, false),
            Direction::Left => (DirectionInner::Right, true),
            Direction::Right => (DirectionInner::Right, false),
            Direction::UpRight => (DirectionInner::DownLeft, true),
            Direction::UpLeft => (DirectionInner::DownRight, true),
            Direction::DownRight => (DirectionInner::DownRight, false),
            Direction::DownLeft => (DirectionInner::DownLeft, false),
        }
    }

    pub fn store(self, x: usize, y: usize) -> (usize, usize, DirectionInner) {
        let (direction_inner, reverse) = self.to_inner();
        if reverse {
            let offset = self.offset();
            (
                (x as isize + offset.0) as usize,
                (y as isize + offset.1) as usize,
                direction_inner,
            )
        } else {
            (x, y, direction_inner)
        }
    }

    pub fn offset(self) -> (isize, isize) {
        match self {
            Direction::Up => (0, -1),
            Direction::UpRight => (1, -1),
            Direction::Right => (1, 0),
            Direction::DownRight => (1, 1),
            Direction::Down => (0, 1),
            Direction::DownLeft => (-1, 1),
            Direction::Left => (-1, 0),
            Direction::UpLeft => (-1, -1),
        }
    }

    // Beveled edges cannot cross each other
    pub fn may_conflict(self, x: usize, y: usize) -> Option<(usize, usize, DirectionInner)> {
        match self {
            Direction::Up => None,
            Direction::Down => None,
            Direction::Left => None,
            Direction::Right => None,
            Direction::UpRight => Some((x, y - 1, DirectionInner::DownRight)),
            Direction::UpLeft => Some((x, y - 1, DirectionInner::DownLeft)),
            Direction::DownRight => Some((x + 1, y, DirectionInner::DownLeft)),
            Direction::DownLeft => Some((x - 1, y, DirectionInner::DownRight)),
        }
    }
}

#[derive(Debug, Clone)]
struct Board {
    board: Vec<(Symbol, u8)>, // simluates a 2d array
    sides: Vec<[Option<Color>; 4]>,
    result: Vec<(usize, usize, Direction, Color)>,
    width: usize,
    height: usize,
}

impl Board {
    #[inline]
    fn index(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    fn add_side(&mut self, x: usize, y: usize, direction: Direction, color: Color) -> bool {
        trace!("try add side ({}, {}, {:?}, {:?})", x, y, direction, color);
        // println!("{:?}", self.board);
        let offset = direction.offset();
        if x as isize + offset.0 < 0
            || x as isize + offset.0 >= self.width as isize
            || y as isize + offset.1 < 0
            || y as isize + offset.1 >= self.height as isize
        {
            // side is out of bounds
            return false;
        }

        if let Some((xc, yc, direction_inner)) = direction.may_conflict(x, y) {
            if self.sides[self.index(xc, yc)][direction_inner as usize].is_some() {
                // crossing with the other beveled edge
                return false;
            }
        }
        let offset_index = self.index(
            (x as isize + offset.0) as usize,
            (y as isize + offset.1) as usize,
        );
        let offset_point = self.board[offset_index];
        if let Symbol::White(n) = offset_point.0 {
            if offset_point.1 + 1 > n {
                // point reach to max number of sides
                return false;
            }
        } else if offset_point.0 == Symbol::color(color)
            || offset_point.0 == Symbol::color_end(color)
        {
            if offset_point.1 > 0 {
                // point already connected
                return false;
            }
        } else {
            // color mismatch
            return false;
        }
        let (xs, ys, direction_inner) = direction.store(x, y);
        let index = self.index(xs, ys);
        let point = self.board[index];
        if !(point.0 == Symbol::color(color)
            || point.0 == Symbol::color_end(color)
            || matches!(point.0, Symbol::White(_)))
        {
            // color mismatch
            return false;
        }
        let side = self.sides[index].get_mut(direction_inner as usize).unwrap();
        if side.is_some() {
            // side already exists
            return false;
        } else {
            *side = Some(color);
        }
        self.board[offset_index].1 += 1;
        self.result.push((x, y, direction, color));
        true
    }

    fn remove_side(&mut self, x: usize, y: usize, direction: Direction) -> bool {
        trace!("remove side ({}, {}, {:?})", x, y, direction);
        let offset = direction.offset();
        let offset_index = self.index(
            (x as isize + offset.0) as usize,
            (y as isize + offset.1) as usize,
        );
        self.board[offset_index].1 -= 1;
        let (x, y, direction_inner) = direction.store(x, y);
        let index = self.index(x, y);
        if self.sides[index][direction_inner as usize].is_none() {
            return false;
        }
        self.sides[index][direction_inner as usize].take();
        self.result.pop();
        true
    }
}

fn solve_color(board: &mut Board, color: Color) -> bool {
    let start = board
        .board
        .iter()
        .position(|&s| s.0 == Symbol::color_end(color));
    if let Some(start_idx) = start {
        info!("solving color {}", color);
        debug!("{:?}", board.board);
        board.board[start_idx].1 += 1;
        let start = (start_idx % board.width, start_idx / board.width);
        let res = solve(board, start, color);
        if !res {
            // backtrack to previous color
            info!("backtrack to previous color");
            board.board[start_idx].1 -= 1;
        }
        res
    } else {
        info!("no start found for color {}", color);
        if let Some(next_color) = color.next() {
            solve_color(board, next_color)
        } else {
            white_solved(board)
        }
    }
}

fn move_to_next_color(board: &mut Board, color: Color) -> bool {
    if let Some(next_color) = color.next() {
        info!("move to next color from {} to {}", color, next_color);
        solve_color(board, next_color)
    } else {
        info!("all color connected");
        white_solved(board)
    }
}

fn solve(board: &mut Board, point: (usize, usize), color: Color) -> bool {
    trace!("solving {:?} at {:?}", color, point);
    for direction in Direction::iter() {
        if board.add_side(point.0, point.1, direction, color) {
            let offset = direction.offset();
            let next_point = (
                (point.0 as isize + offset.0) as usize,
                (point.1 as isize + offset.1) as usize,
            );
            if board.board[board.index(next_point.0, next_point.1)].0 == Symbol::color_end(color) {
                if color_solved(board, color) {
                    info!("solved color {:?}", color);
                    if move_to_next_color(board, color) {
                        return true;
                    } // else continue to solve this color
                } else {
                    trace!("color {:?} reach to end but not all connected", color);
                }
            } else {
                let result = solve(board, next_point, color);
                if result {
                    return true;
                }
            }
            board.remove_side(point.0, point.1, direction);
        }
    }
    false
}

fn color_solved(board: &Board, color: Color) -> bool {
    let mut board_clone = board.board.iter().map(|s| s.0).collect::<Vec<_>>();
    for (i, side) in board.sides.iter().enumerate() {
        for (dir, side_color) in side.iter().enumerate() {
            if let Some(side_color) = side_color {
                if *side_color == color {
                    let direction: Direction = DirectionInner::from(dir as u8).into();
                    let x = i % board.width;
                    let y = i / board.width;
                    let offset = direction.offset();

                    board_clone[i] = Symbol::Empty;
                    let index = board.index(
                        (x as isize + offset.0) as usize,
                        (y as isize + offset.1) as usize,
                    );
                    board_clone[index] = Symbol::Empty;
                }
            }
        }
    }

    !board_clone.contains(&Symbol::color(color))
}

fn white_solved(board: &Board) -> bool {
    for (c, count) in board.board.iter() {
        if let Symbol::White(n) = c {
            if n != count {
                info!("white not solved, backtrack");
                return false;
            }
        }
    }
    true
}

fn main() {
    pretty_env_logger::init();
    let mut lines = io::stdin().lines();
    let Some(fist_line) = lines.next() else {
        error!("no input");
        return;
    };
    let fist_line = fist_line.unwrap();
    let length = fist_line.len();
    let mut board = Vec::new();
    for c in fist_line.chars() {
        board.push(Symbol::from(c));
    }
    for line in lines {
        let line = line.unwrap();
        if line.len() != length {
            error!("current line length is not equal to the first line length");
            return;
        }
        for c in line.chars() {
            board.push(Symbol::from(c));
        }
    }

    let counts = board.iter().counts();
    if counts.contains_key(&Symbol::REnd) && counts[&Symbol::REnd] != 2 {
        error!(
            "There are {} R endpoints, but there should be 0 or 2",
            counts[&Symbol::REnd]
        );
        return;
    }
    if counts.contains_key(&Symbol::GEnd) && counts[&Symbol::GEnd] != 2 {
        error!(
            "There are {} G endpoints, but there should be 0 or 2",
            counts[&Symbol::GEnd]
        );
        return;
    }
    if counts.contains_key(&Symbol::BEnd) && counts[&Symbol::BEnd] != 2 {
        error!(
            "There are {} B endpoints, but there should be 0 or 2",
            counts[&Symbol::BEnd]
        );
        return;
    }

    warn!("start solving");

    let now = std::time::Instant::now();

    let width = length;
    let height = board.len() / width;
    let mut sides = Vec::new();
    for _ in 0..board.len() {
        sides.push([None; 4]);
    }

    let mut board = Board {
        board: board.into_iter().map(|s| (s, 0)).collect(),
        sides,
        result: Vec::new(),
        width,
        height,
    };

    let res = solve_color(&mut board, Color::Red);
    if res {
        info!("solution found");
        debug!("{:?}", board.board);
        debug!("{:?}", board.sides);
        for (color, group) in &board.result.iter().group_by(|s| s.3) {
            println!("{}:", color);
            for (x, y, direction, _) in group {
                println!("{} {} {}", direction, x, y);
            }
        }
    } else {
        warn!("no solution");
    }

    let elapsed_time = now.elapsed();
    println!("Running takes {} seconds.", elapsed_time.as_secs());
}
