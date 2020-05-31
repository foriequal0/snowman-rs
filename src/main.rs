use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashSet, VecDeque};
use std::fmt::{Display, Formatter};
use std::io::{stdin, Read};
use std::str::FromStr;
use std::thread::sleep;
use std::time::Duration;

fn main() {
    let mut buf = Vec::new();
    stdin().read_to_end(&mut buf).unwrap();
    let lines: Vec<_> = std::str::from_utf8(&buf).unwrap().lines().collect();
    let initlal = State::from_lines(&lines);
    let solved = solve(initlal.clone()).unwrap();

    let mut state = initlal;
    for (index, direction) in solved.directions {
        print!("{esc}[2J{esc}[1;1H", esc = 27 as char); // clear and position cursor 1,1
        print!("{}", state);
        sleep(Duration::from_secs(1));
        state.step_ball(index, direction);
    }
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char); // clear and position cursor 1,1
    print!("{}", state);
}

fn solve(state: State) -> Result<State, &'static str> {
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    queue.push_back(state);
    while let Some(state) = queue.pop_front() {
        if state
            .balls
            .iter()
            .all(|ball| ball.pos == state.balls[0].pos)
        {
            return Ok(state);
        }
        let concise = state.concise();
        if visited.contains(&concise) {
            continue;
        }
        visited.insert(concise);
        for ball_idx in 0..state.balls.len() {
            for dir in vec![
                Direction::Right,
                Direction::Left,
                Direction::Down,
                Direction::Up,
            ] {
                let mut state = state.clone();
                if state.step_ball(ball_idx, dir) {
                    queue.push_back(state.clone());
                }
            }
        }
    }

    return Err("Not solvable");
}

#[derive(Clone)]
struct State {
    width: usize,
    height: usize,
    ground: Vec<Ground>,
    balls: Vec<Ball>,
    player: (i32, i32),
    directions: Vec<(usize, Direction)>,
}

#[derive(Clone, Eq, PartialEq, Hash)]
struct Concise {
    ground: Vec<Ground>,
    balls: Vec<Ball>,
    fill: Vec<bool>,
}

impl State {
    fn from_lines(lines: &[&str]) -> State {
        let player_x = i32::from_str(lines[0]).expect("invalid x position");
        let player_y = i32::from_str(lines[1]).expect("invalid y position");
        let mut balls = Vec::new();
        let mut ground_lines = Vec::new();
        for (y, line) in lines[2..].iter().enumerate() {
            let mut ground_line = Vec::new();
            for (x, char) in line.trim().chars().enumerate() {
                let ground = match char {
                    '.' => Ground::None,
                    '_' => Ground::Snow,
                    '#' => Ground::Block,
                    '1' | '2' | '4' => {
                        balls.push(Ball {
                            size: char.to_digit(10).unwrap() as u8,
                            pos: (x as i32, y as i32),
                        });
                        Ground::None
                    }
                    _ => panic!("invalid char"),
                };
                ground_line.push(ground);
            }
            ground_lines.push(ground_line);
        }
        State {
            width: ground_lines[0].len(),
            height: ground_lines.len(),
            ground: ground_lines.into_iter().flatten().collect(),
            balls,
            player: (player_x, player_y),
            directions: Vec::new(),
        }
    }

    #[inline]
    fn get(&self, x: i32, y: i32) -> Option<Ground> {
        if x < 0 || x >= self.width as i32 || y < 0 || y >= self.height as i32 {
            return None;
        }
        Some(self.ground[self.width * y as usize + x as usize])
    }

    #[inline]
    fn set(&mut self, x: usize, y: usize, value: Ground) {
        self.ground[self.width * y + x] = value;
    }

    // A* on taxi metric
    fn move_to(&mut self, target: (i32, i32)) -> bool {
        #[derive(PartialEq, Eq, Ord)]
        struct Item {
            pos: (i32, i32),
            sunk_cost: i32,
            heuristic_cost: i32,
        };
        impl PartialOrd for Item {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                (self.sunk_cost + self.heuristic_cost)
                    .partial_cmp(&(other.sunk_cost + other.heuristic_cost))
            }
        }

        let mut queue = BinaryHeap::with_capacity(self.width * self.height);
        let mut visited = HashSet::with_capacity(self.width * self.height);
        fn push(queue: &mut BinaryHeap<Item>, target: (i32, i32), sunk_cost: i32, pos: (i32, i32)) {
            queue.push(Item {
                pos,
                sunk_cost,
                heuristic_cost: (target.0 - pos.0).abs() + (target.1 - pos.1).abs(),
            });
        }
        push(
            &mut queue,
            target,
            0,
            (self.player.0 as i32, self.player.1 as i32),
        );

        while let Some(Item { pos, sunk_cost, .. }) = queue.pop() {
            if visited.contains(&pos) {
                continue;
            }
            if matches!(self.get(pos.0, pos.1), Some(Ground::Block) | None) {
                continue;
            }
            if (self.balls.iter()).any(|ball| ball.pos == pos) {
                continue;
            }

            if target == pos {
                self.player = target;
                return true;
            }
            visited.insert(pos);
            push(
                &mut queue,
                target,
                sunk_cost + 1,
                (pos.0 as i32 - 1, pos.1 as i32),
            );
            push(
                &mut queue,
                target,
                sunk_cost + 1,
                (pos.0 as i32 + 1, pos.1 as i32),
            );
            push(
                &mut queue,
                target,
                sunk_cost + 1,
                (pos.0 as i32, pos.1 as i32 - 1),
            );
            push(
                &mut queue,
                target,
                sunk_cost + 1,
                (pos.0 as i32, pos.1 as i32 + 1),
            );
        }
        false
    }

    fn push(&mut self, ball_idx: usize, dir: Direction) -> bool {
        let this = self.balls[ball_idx];
        let (x, y) = (this.pos.0 as i32, this.pos.1 as i32);
        let (nx, ny) = dir.step(x, y);
        // you can't put the snowball on the chasm or a block
        if matches!(self.get(nx, ny), None | Some(Ground::Block)) {
            return false;
        }
        // you can't put a larger ball on top of a smaller ball
        let smaller_ball = (self.balls.iter().enumerate())
            .any(|(idx, ball)| idx != ball_idx && ball.pos == (nx, ny) && ball.size <= this.size);
        if smaller_ball {
            return false;
        }
        // you can't push a larger ball if a smaller ball is on top of it
        let ball_on_top = (self.balls.iter().enumerate())
            .any(|(idx, ball)| idx != ball_idx && ball.pos == (x, y) && ball.size < this.size);
        if ball_on_top {
            return false;
        }

        self.balls[ball_idx].pos = (nx, ny);
        if matches!(self.get(nx, ny), Some(Ground::Snow)) {
            if self.balls[ball_idx].size < 4 {
                self.balls[ball_idx].size *= 2;
            }
        }
        self.set(nx as usize, ny as usize, Ground::None);

        let any_ball = (self.balls.iter().enumerate())
            .any(|(idx, ball)| idx != ball_idx && ball.pos == (x, y));
        if !any_ball {
            self.player = (x, y);
        }

        true
    }

    fn step_ball(&mut self, ball_idx: usize, dir: Direction) -> bool {
        let (x, y) = self.balls[ball_idx].pos;
        if !self.move_to(dir.inverse().step(x as i32, y as i32)) {
            return false;
        }
        if !self.push(ball_idx, dir) {
            return false;
        }
        self.directions.push((ball_idx, dir));
        true
    }

    fn concise(&self) -> Concise {
        let mut queue = VecDeque::with_capacity(self.width * self.height);
        queue.push_back(self.player);
        let mut fill = vec![false; self.width * self.height];
        while let Some(pos) = queue.pop_front() {
            let offset = pos.0 as usize + pos.1 as usize * self.width;
            if fill[offset] {
                continue;
            }
            fill[offset] = true;
        }
        Concise {
            ground: self.ground.clone(),
            balls: self.balls.clone(),
            fill,
        }
    }
}

impl Display for State {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        for y in 0..self.height as i32 {
            for x in 0..self.width as i32 {
                if (x, y) == self.player {
                    write!(f, "A")?;
                } else {
                    let mut sum = 0;
                    for ball in &self.balls {
                        if (x, y) == ball.pos {
                            sum += ball.size;
                        }
                    }
                    if sum != 0 {
                        write!(f, "{}", sum)?;
                    } else {
                        write!(f, " ")?;
                    }
                }
                match self.get(x as i32, y as i32).unwrap() {
                    Ground::None => write!(f, ".")?,
                    Ground::Snow => write!(f, "_")?,
                    Ground::Block => write!(f, "#")?,
                };
            }
            writeln!(f)?;
        }
        for (id, dir) in &self.directions {
            writeln!(f, "{}\t{:?}", id, dir)?;
        }
        Ok(())
    }
}

#[derive(Eq, PartialEq, Copy, Clone, Hash)]
enum Ground {
    None,
    Snow,
    Block,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
struct Ball {
    size: u8,
    pos: (i32, i32),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Direction {
    Left,
    Right,
    Down,
    Up,
}

impl Direction {
    fn inverse(&self) -> Direction {
        match self {
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
            Direction::Down => Direction::Up,
            Direction::Up => Direction::Down,
        }
    }

    fn step(&self, x: i32, y: i32) -> (i32, i32) {
        match self {
            Direction::Left => (x - 1, y),
            Direction::Right => (x + 1, y),
            Direction::Up => (x, y - 1),
            Direction::Down => (x, y + 1),
        }
    }
}
