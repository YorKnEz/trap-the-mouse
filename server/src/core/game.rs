use serde_derive::Serialize;

pub const GRID_SIZE: usize = 11;

// position tuples have the following meaning .0 - line, .1 - column

#[derive(Clone, Serialize)]
pub struct GameState {
    pub angel: u32, // id of the user that is the angel, if 0 it's the computer
    pub devil: u32, // id of the user that is the devil
    pub devil_pos: (i32, i32),
    pub turn: bool,                           // true - angel, false - devil
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
        let devil_pos = (GRID_SIZE as i32 / 2, GRID_SIZE as i32 / 2);

        let mut grid = [[false; GRID_SIZE]; GRID_SIZE];

        for i in 0..GRID_SIZE {
            for j in 0..GRID_SIZE {
                if i as i32 != devil_pos.0 && j as i32 != devil_pos.1 {
                    grid[i][j] = (rand::random::<u32>() % 100) < 12;
                }
            }
        }

        GameState {
            angel,
            devil,
            devil_pos,
            turn: false,
            grid,
        }
    }

    pub fn contains(&self, pos: (i32, i32)) -> bool {
        0 <= pos.0 && pos.0 < GRID_SIZE as i32 && 0 <= pos.1 && pos.1 < GRID_SIZE as i32
    }

    pub fn valid_devil_move(&self, pos: (i32, i32)) -> bool {
        if !self.contains(pos) {
            return false;
        }

        if self.grid[pos.0 as usize][pos.1 as usize] {
            return false;
        }

        for off in GameState::D[(self.devil_pos.0 % 2) as usize] {
            let new_pos = (self.devil_pos.0 + off.0, self.devil_pos.1 + off.1);
            if self.contains(new_pos) && new_pos == pos {
                return true;
            }
        }

        return false;
    }

    pub fn devil_won(&self) -> bool {
        self.reached_border(self.devil_pos)
    }

    fn reached_border(&self, pos: (i32, i32)) -> bool {
        pos.0 == 0 || pos.0 == GRID_SIZE as i32 - 1 || pos.1 == 0 || pos.1 == GRID_SIZE as i32 - 1
    }

    pub fn angel_won(&self) -> bool {
        let mut visited = [[false; GRID_SIZE]; GRID_SIZE];

        !self.border_reachable(self.devil_pos, &mut visited)
    }

    fn border_reachable(
        &self,
        pos: (i32, i32),
        visited: &mut [[bool; GRID_SIZE]; GRID_SIZE],
    ) -> bool {
        visited[pos.0 as usize][pos.1 as usize] = true;

        if self.reached_border(pos) && !self.grid[pos.0 as usize][pos.1 as usize] {
            return true;
        }

        let mut res = false;

        for off in GameState::D[(pos.0 % 2) as usize] {
            let new_pos = (pos.0 + off.0, pos.1 + off.1);
            // check if new_pos is in bounds, not blocked and not visited
            if self.contains(new_pos)
                && !self.grid[pos.0 as usize][pos.1 as usize]
                && !visited[new_pos.0 as usize][new_pos.1 as usize]
            {
                res = res || self.border_reachable(new_pos, visited);
            }
        }

        return res;
    }
}
