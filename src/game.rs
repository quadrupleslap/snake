#![allow(dead_code)]//TODO: Remove.

use core::ops;

type Position = [usize; 2];

struct Game {
    /// The game board.
    board: Board,
    /// The blue player.
    blue: Player,
    /// The red player.
    red: Player,
    /// The time for which the current state has been active.
    time: usize,
    /// The current state.
    state: State,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum State {
    /// [Space] → Countdown
    Title,
    /// 3 → 2 → 1 → Main
    Countdown(usize),
    /// [P] → Pause, End → Death
    Main,
    /// End → Countdown
    Death,
    /// [P] → Main, [Q] → Title
    Paused,
}

struct Player {
    alive: bool,
    score: u32,
    direction: Direction,
    pos: Position,
}

struct Board {
    grid: [[Cell; 50]; 25],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Cell {
    Empty,
    Red,
    Blue,
}

impl Game {
    pub fn new() -> Self {
        Self {
            board: Board::new([12, 16], [12, 33]),
            blue: Player::new([12, 16], Direction::Right),
            red: Player::new([12, 33], Direction::Left),
            time: 0,
            state: State::Title,
        }
    }

    pub fn tick(&mut self) {
        self.time += 1;

        match self.state {
            State::Title => {
                // Do nothing.
            }
            State::Countdown(ref mut n) => {
                if self.time % 10 == 0 {
                    if *n == 0 {
                        self.set_state(State::Main);
                    } else {
                        *n -= 1;
                    }
                }
            }
            State::Main => {
                // Grow the wall.

                self.board[self.blue.pos] = Cell::Blue;
                self.board[self.red.pos] = Cell::Red;

                // Move the players.

                self.blue.step();
                self.red.step();

                // Collide blue against the walls.

                if self.board[self.blue.pos] != Cell::Empty {
                    self.blue.alive = false;
                    self.red.score += 1;
                }

                // Collide red against the walls.

                if self.board[self.red.pos] != Cell::Empty {
                    self.red.alive = false;
                    self.blue.score += 1;
                }

                // Collide blue against red.

                if self.red.pos == self.blue.pos {
                    self.blue.alive = false;
                    self.blue.score += 1;
                    self.red.alive = false;
                    self.red.score += 1;
                }

                // Go to death if at least one of them is dead.

                if !(self.blue.alive && self.red.alive) {
                    self.set_state(State::Death);
                }
            }
            State::Death => {
                //TODO: Animate the death.
                self.set_state(State::Countdown(3));
            }
            State::Paused => {
                // Do nothing.
            }
        }
    }

    /// Press an arrow key.
    pub fn input(&mut self, red: bool, d: Direction) {
        if self.state != State::Main { return }

        if red {
            self.red.direction = d;
        } else {
            self.blue.direction = d;
        }
    }

    /// Press the start button.
    pub fn start(&mut self) {
        if self.state != State::Title { return }
        self.set_state(State::Countdown(3));
    }

    /// Press the pause button.
    pub fn pause(&mut self) {
        if self.state == State::Main {
            self.set_state(State::Paused);
        } else if self.state == State::Paused {
            self.set_state(State::Main);
        }
    }

    /// Press the quit button.
    pub fn quit(&mut self) {
        if let State::Paused { .. } = self.state {
            self.state = State::Title;
        }
    }

    fn set_state(&mut self, state: State) {
        self.state = state;
        self.time = 0;
    }
}

impl Board {
    fn new(blue: Position, red: Position) -> Self {
        let mut board = Self { grid: [[Cell::Empty; 50]; 25] };
        board[blue] = Cell::Blue;
        board[red] = Cell::Red;
        board
    }
}

impl Player {
    fn new(pos: Position, direction: Direction) -> Self {
        Self {
            alive: true,
            score: 0,
            direction,
            pos,
        }
    }

    fn step(&mut self) {
        match self.direction {
            Direction::Up => self.pos[0] = (self.pos[0] + 24) % 25,
            Direction::Down => self.pos[0] = (self.pos[0] + 1) % 25,
            Direction::Left => self.pos[1] = (self.pos[1] + 49) % 50,
            Direction::Right => self.pos[1] = (self.pos[1] + 1) % 50,
        }
    }
}

impl ops::Index<Position> for Board {
    type Output = Cell;
    fn index(&self, p: Position) -> &Cell {
        &self.grid[p[0]][p[1]]
    }
}

impl ops::IndexMut<Position> for Board {
    fn index_mut(&mut self, p: Position) -> &mut Cell {
        &mut self.grid[p[0]][p[1]]
    }
}
