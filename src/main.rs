use iter_tools::Itertools;
use log::{debug, error, info, trace, warn};
use std::io;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};

type Point = (i32, i32);

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

// Puzzle nodes
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

// Only store these 4 directions
// The other 4 are just the reverse of these
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

    pub fn store(self, pos: Point) -> (Point, DirectionInner) {
        let (direction_inner, reverse) = self.to_inner();
        if reverse {
            (self.apply_offset(pos), direction_inner)
        } else {
            (pos, direction_inner)
        }
    }

    pub fn offset(self) -> (i32, i32) {
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

    pub fn apply_offset(self, (x, y): Point) -> Point {
        let offset = self.offset();
        (x + offset.0, y + offset.1)
    }

    // Beveled edges cannot cross each other
    pub fn may_conflict(self, (x, y): Point) -> Option<(Point, DirectionInner)> {
        match self {
            Direction::Up => None,
            Direction::Down => None,
            Direction::Left => None,
            Direction::Right => None,
            Direction::UpRight => Some(((x, y - 1), DirectionInner::DownRight)),
            Direction::UpLeft => Some(((x, y - 1), DirectionInner::DownLeft)),
            Direction::DownRight => Some(((x + 1, y), DirectionInner::DownLeft)),
            Direction::DownLeft => Some(((x - 1, y), DirectionInner::DownRight)),
        }
    }
}

#[derive(Debug, Clone)]
struct Board {
    board: Vec<(Symbol, u8)>, // simluates a 2d array
    width: usize,
    height: usize,
    lines: Vec<[Option<Color>; 4]>, // store the currect state of conneced lines, index by start position of the line
    result: Vec<(Point, Direction, Color)>,
}

impl Board {
    // convert a point to a index
    #[inline]
    fn index(&self, (x, y): Point) -> usize {
        (y * self.width as i32 + x) as usize
    }

    // convert a index to a point
    #[inline]
    fn pos(&self, index: usize) -> Point {
        ((index % self.width) as i32, (index / self.width) as i32)
    }

    // add a connected line to the board if it is legal
    //
    // return whether the line is legal
    fn add_line(&mut self, start_pos: Point, direction: Direction, color: Color) -> bool {
        trace!("try add line ({:?}, {}, {})", start_pos, direction, color);
        // println!("{:?}", self.board);
        let offset_pos = direction.apply_offset(start_pos);
        if offset_pos.0 < 0
            || offset_pos.0 >= self.width as i32
            || offset_pos.1 < 0
            || offset_pos.1 >= self.height as i32
        {
            // line is out of bounds
            return false;
        }

        if let Some((conflict_point, direction_inner)) = direction.may_conflict(start_pos) {
            if self.lines[self.index(conflict_point)][direction_inner as usize].is_some() {
                // crossing with the other beveled edge
                return false;
            }
        }
        let offset_index = self.index(offset_pos);
        let offset_point = self.board[offset_index];
        if let Symbol::White(n) = offset_point.0 {
            if offset_point.1 + 1 > n {
                // point reach to max number of lines
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
        let (store_pos, direction_inner) = direction.store(start_pos);
        let index = self.index(store_pos);
        let point = self.board[index];
        if !(point.0 == Symbol::color(color)
            || point.0 == Symbol::color_end(color)
            || matches!(point.0, Symbol::White(_)))
        {
            // color mismatch
            return false;
        }
        let line = self.lines[index].get_mut(direction_inner as usize).unwrap();
        if line.is_some() {
            // line already exists
            return false;
        } else {
            *line = Some(color);
        }
        self.board[offset_index].1 += 1;
        self.result.push((start_pos, direction, color));
        true
    }

    // remove a connected line from the board
    fn remove_line(&mut self, start_pos: Point, direction: Direction) -> bool {
        trace!("remove line ({:?}, {:?})", start_pos, direction);
        let offset_index = self.index(direction.apply_offset(start_pos));
        self.board[offset_index].1 -= 1;
        let (store_pos, direction_inner) = direction.store(start_pos);
        let store_index = self.index(store_pos);
        if self.lines[store_index][direction_inner as usize].is_none() {
            return false;
        }
        self.lines[store_index][direction_inner as usize].take();
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
        let start = board.pos(start_idx);
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

fn solve(board: &mut Board, point: (i32, i32), color: Color) -> bool {
    trace!("solving {:?} at {:?}", color, point);
    for direction in Direction::iter() {
        if board.add_line(point, direction, color) {
            let next_point = direction.apply_offset(point);
            if board.board[board.index(next_point)].0 == Symbol::color_end(color) {
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
            board.remove_line(point, direction);
        }
    }
    false
}

fn color_solved(board: &Board, color: Color) -> bool {
    let mut board_clone = board.board.iter().map(|s| s.0).collect::<Vec<_>>();
    for (i, line) in board.lines.iter().enumerate() {
        for (dir, line_color) in line.iter().enumerate() {
            if let Some(line_color) = line_color {
                if *line_color == color {
                    let direction: Direction = DirectionInner::from(dir as u8).into();
                    let pos = board.pos(i);

                    let i2 = board.index(direction.apply_offset(pos));
                    board_clone[i] = Symbol::Empty;
                    board_clone[i2] = Symbol::Empty;
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
    let mut lines = Vec::new();
    for _ in 0..board.len() {
        lines.push([None; 4]);
    }

    let mut board = Board {
        board: board.into_iter().map(|s| (s, 0)).collect(),
        lines,
        result: Vec::new(),
        width,
        height,
    };

    let res = solve_color(&mut board, Color::Red);
    if res {
        info!("solution found");
        debug!("{:?}", board.board);
        debug!("{:?}", board.lines);
        for (color, group) in &board.result.iter().group_by(|s| s.2) {
            println!("{}:", color);
            for (point, direction, _) in group {
                println!("{} {:?}", direction, point);
            }
        }
    } else {
        warn!("no solution");
    }

    let elapsed_time = now.elapsed();
    println!("Running takes {} seconds.", elapsed_time.as_secs());
}
