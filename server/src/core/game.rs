use std::collections::VecDeque;

use serde_derive::Serialize;

pub const GRID_SIZE: usize = 11;

// position tuples have the following meaning .0 - line, .1 - column

#[derive(Clone, Serialize)]
pub struct GameState {
    pub devil: u32, // id of the user that is the devil
    pub angel: u32, // id of the user that is the nagel, if 0 it's the computer
    pub angel_pos: (i32, i32),
    pub turn: bool,                           // false - angel, true - devil
    pub grid: [[bool; GRID_SIZE]; GRID_SIZE], // whether the tile is blocked or not
}

#[derive(Debug, Serialize)]
pub struct GameUpdate {
    pub win: (bool, bool), // (devil won, angel won)
    pub turn: bool,
    pub user_move: (i32, i32),
}

impl GameState {
    const D: [[(i32, i32); 6]; 2] = [
        [(-1, 0), (0, 1), (1, 0), (1, -1), (0, -1), (-1, -1)],
        [(-1, 0), (-1, 1), (0, 1), (1, 1), (1, 0), (0, -1)],
    ];

    pub fn new(angel: u32, devil: u32) -> GameState {
        let angel_pos = (GRID_SIZE as i32 / 2, GRID_SIZE as i32 / 2);

        let mut grid = [[false; GRID_SIZE]; GRID_SIZE];

        for (i, line) in grid.iter_mut().enumerate() {
            for (j, item) in line.iter_mut().enumerate() {
                if i as i32 != angel_pos.0 && j as i32 != angel_pos.1 {
                    *item = (rand::random::<u32>() % 100) < 12;
                }
            }
        }

        GameState {
            devil,
            angel,
            angel_pos,
            turn: true,
            grid,
        }
    }

    pub fn contains(&self, pos: (i32, i32)) -> bool {
        0 <= pos.0 && pos.0 < GRID_SIZE as i32 && 0 <= pos.1 && pos.1 < GRID_SIZE as i32
    }

    pub fn valid_angel_move(&self, pos: (i32, i32)) -> bool {
        if !self.contains(pos) {
            return false;
        }

        if self.grid[pos.0 as usize][pos.1 as usize] {
            return false;
        }

        for off in GameState::D[(self.angel_pos.0 % 2) as usize] {
            let new_pos = (self.angel_pos.0 + off.0, self.angel_pos.1 + off.1);
            if self.contains(new_pos) && new_pos == pos {
                return true;
            }
        }

        false
    }

    pub fn angel_won(&self) -> bool {
        self.reached_border(self.angel_pos)
    }

    fn reached_border(&self, pos: (i32, i32)) -> bool {
        pos.0 == 0 || pos.0 == GRID_SIZE as i32 - 1 || pos.1 == 0 || pos.1 == GRID_SIZE as i32 - 1
    }

    pub fn find_path(&self) -> Option<(i32, i32)> {
        // if the angel reached the border there is no point in finding a path
        if self.reached_border(self.angel_pos) {
            return Some(self.angel_pos);
        }

        let mut pos = self.angel_pos;
        let mut q = VecDeque::new();
        let mut len = [[0; GRID_SIZE]; GRID_SIZE];

        let mut path = Vec::new();

        q.push_back(pos);
        len[pos.0 as usize][pos.1 as usize] = 1;

        while let Some(pos) = q.pop_front() {
            if self.reached_border(pos) {
                path.push(pos);
                break;
            }

            // try random directions
            let mut offsets = Vec::from(GameState::D[(pos.0 % 2) as usize]);

            while !offsets.is_empty() {
                let i = rand::random::<usize>() % offsets.len();
                let off = offsets.remove(i);

                let new_pos = (pos.0 + off.0, pos.1 + off.1);
                // check if new_pos is in bounds, not blocked and not visited
                if self.contains(new_pos)
                    && !self.grid[new_pos.0 as usize][new_pos.1 as usize]
                    && len[new_pos.0 as usize][new_pos.1 as usize] == 0
                {
                    q.push_back(new_pos);
                    len[new_pos.0 as usize][new_pos.1 as usize] =
                        len[pos.0 as usize][pos.1 as usize] + 1;
                }
            }
        }

        if path.is_empty() {
            return None;
        }

        let mut found = false;
        pos = *path.last().unwrap();

        while pos != self.angel_pos {
            for off in GameState::D[(pos.0 % 2) as usize] {
                let new_pos = (pos.0 + off.0, pos.1 + off.1);
                // check if new_pos is in bounds, not blocked and not visited
                if self.contains(new_pos)
                    && !self.grid[new_pos.0 as usize][new_pos.1 as usize]
                    && len[pos.0 as usize][pos.1 as usize]
                        == len[new_pos.0 as usize][new_pos.1 as usize] + 1
                {
                    path.push(new_pos);
                    found = true;
                    break;
                }
            }

            if !found {
                return None;
            }

            pos = *path.last().unwrap();
        }

        Some(path[path.len() - 2])
    }
}
