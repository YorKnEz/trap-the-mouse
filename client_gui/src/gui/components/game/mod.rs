mod tile;

use std::sync::mpsc;

use sfml::{
    graphics::{Color, Drawable, FloatRect, RectangleShape, Shape, Sprite, Texture, Transformable},
    system::Vector2f,
    SfBox,
};

use crate::{
    events::{EventData, GameMoveEventData, GameStartedEvent, GameUpdatedEvent, UIEvent, Window},
    types::GRID_SIZE,
};

use self::tile::Tile;

use super::{Clickable, Fixed};

pub struct Game {
    event_data: EventData,
    bounds: FloatRect,
    tile_texture: SfBox<Texture>,
    wall_texture: SfBox<Texture>,
    player_texture: SfBox<Texture>,

    grid: [[Tile; GRID_SIZE]; GRID_SIZE],
    player_pos: (usize, usize),
    ratio: f32,
    sender: mpsc::Sender<UIEvent>,
    pub began: bool,
}

impl Game {
    const REAL_WIDTH: f32 = 64.0;

    pub fn new(
        id: u32,
        window: Window,
        left: f32,
        top: f32,
        width: f32,
        height: f32,
        sender: mpsc::Sender<UIEvent>,
    ) -> Game {
        let bounds = FloatRect {
            left,
            top,
            width,
            height,
        };

        let tile_width = bounds.width / (2 * GRID_SIZE + 1) as f32;
        let ratio = tile_width / Game::REAL_WIDTH;

        let off = [
            Vector2f::new(0.0, -tile_width),
            Vector2f::new(tile_width, -(tile_width / 2.0)),
            Vector2f::new(tile_width, tile_width / 2.0),
            Vector2f::new(0.0, tile_width),
            Vector2f::new(-tile_width, tile_width / 2.0),
            Vector2f::new(-tile_width, -(tile_width / 2.0)),
        ];

        let mut grid = [[Default::default(); GRID_SIZE]; GRID_SIZE];

        for i in 0..GRID_SIZE {
            for j in 0..GRID_SIZE {
                let origin = Vector2f::new(
                    bounds.left
                        + tile_width
                        + j as f32 * (2.0 * tile_width)
                        + (i % 2) as f32 * tile_width,
                    bounds.top + 2.0 * tile_width + i as f32 * (1.5 * tile_width),
                );

                grid[i][j] = Tile::new(origin, &off);
            }
        }

        Game {
            event_data: EventData { id, window },
            bounds,
            tile_texture: Texture::from_file("./client_gui/assets/tile.png").unwrap(),
            wall_texture: Texture::from_file("./client_gui/assets/wall.png").unwrap(),
            player_texture: Texture::from_file("./client_gui/assets/player.png").unwrap(),
            grid,
            player_pos: (GRID_SIZE / 2, GRID_SIZE / 2),
            ratio,
            sender,
            began: false,
        }
    }

    pub fn start(&mut self, state: GameStartedEvent) {
        self.began = true;
        self.player_pos = (state.devil_pos.0 as usize, state.devil_pos.1 as usize);

        for i in 0..GRID_SIZE {
            for j in 0..GRID_SIZE {
                self.grid[i][j].set_blocked(state.grid[i][j]);
            }
        }
    }

    pub fn stop(&mut self) {
        self.began = false;
        self.player_pos = (GRID_SIZE / 2, GRID_SIZE / 2);

        for i in 0..GRID_SIZE {
            for j in 0..GRID_SIZE {
                self.grid[i][j].set_blocked(false);
            }
        }
    }

    pub fn update(&mut self, state: GameUpdatedEvent) {
        let user_move = (state.user_move.0 as usize, state.user_move.1 as usize);
        // devil move
        if state.turn == false {
            self.player_pos = (user_move.0, user_move.1);
        }
        // angel move
        else {
            self.grid[user_move.0][user_move.1].set_blocked(true);
        }

        if state.win.0 || state.win.1 {
            self.stop();
        }
    }

    pub fn click(&self, x: u32, y: u32) {
        println!("{x} {y}");
        for i in 0..GRID_SIZE {
            for j in 0..GRID_SIZE {
                if self.grid[i][j].inside(x, y) {
                    println!("{i} {j}");
                    if let Err(e) = self.sender.send(UIEvent::GameMove(GameMoveEventData {
                        x: i as i32,
                        y: j as i32,
                    })) {
                        println!("send error: {e:?}");
                    }
                }
            }
        }
    }
}

impl Clickable for Game {
    fn get_id(&self) -> u32 {
        self.event_data.id
    }

    fn click(&mut self, x: u32, y: u32) {
        Game::click(self, x, y);
    }

    fn no_click(&mut self) {}
}

impl Fixed for Game {
    fn bounds(&self) -> FloatRect {
        self.bounds
    }

    fn position(&self) -> Vector2f {
        (self.bounds.left, self.bounds.top).into()
    }

    fn set_position(&mut self, position: Vector2f) {
        let old_pos = self.position();
        let offset = Vector2f::new(position.x - old_pos.x, position.y - old_pos.y);

        self.bounds.left = position.x;
        self.bounds.top = position.y;

        for i in 0..GRID_SIZE {
            for j in 0..GRID_SIZE {
                self.grid[i][j].origin.x += offset.x;
                self.grid[i][j].origin.y += offset.y;

                for point in self.grid[i][j].points.iter_mut() {
                    point.x += offset.x;
                    point.y += offset.y;
                }
            }
        }
    }
}

impl Drawable for Game {
    fn draw<'a: 'shader, 'texture, 'shader, 'shader_texture>(
        &'a self,
        target: &mut dyn sfml::graphics::RenderTarget,
        _: &sfml::graphics::RenderStates<'texture, 'shader, 'shader_texture>,
    ) {
        for i in 0..GRID_SIZE {
            for j in 0..GRID_SIZE {
                let mut sprite = Sprite::with_texture(&self.tile_texture);
                sprite.set_origin((Game::REAL_WIDTH, Game::REAL_WIDTH));
                sprite.set_scale((self.ratio, self.ratio));
                sprite.set_position(self.grid[i][j].origin);
                target.draw(&sprite);

                if self.grid[i][j].blocked {
                    let mut sprite = Sprite::with_texture(&self.wall_texture);
                    sprite.set_origin((Game::REAL_WIDTH, 2.0 * Game::REAL_WIDTH));
                    sprite.set_scale((self.ratio, self.ratio));
                    sprite.set_position(self.grid[i][j].origin);
                    target.draw(&sprite);
                }

                if i == self.player_pos.0 && j == self.player_pos.1 {
                    let mut sprite = Sprite::with_texture(&self.player_texture);
                    sprite.set_origin((Game::REAL_WIDTH, Game::REAL_WIDTH));
                    sprite.set_scale((self.ratio, self.ratio));
                    sprite.set_position(self.grid[self.player_pos.0][self.player_pos.1].origin);
                    target.draw(&sprite);
                }
            }
        }

        let mut cover = RectangleShape::new();
        if self.began {
            cover.set_fill_color(Color::TRANSPARENT);
        } else {
            cover.set_fill_color(Color::rgba(0, 0, 0, 200));
        }
        cover.set_size((self.bounds.width, self.bounds.height));
        cover.set_position((self.bounds.left, self.bounds.top));
        cover.set_outline_color(Color::BLACK);
        cover.set_outline_thickness(2.0);
        target.draw(&cover);
    }
}
