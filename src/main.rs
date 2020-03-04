extern crate tetra;
extern crate rustbreak;
extern crate ron;
extern crate rand;
#[macro_use]
extern crate log;
extern crate env_logger;
use tetra::graphics::{Texture, Color};
use tetra::{ContextBuilder, State, Context, Event, graphics};
use tetra::graphics::{draw, Camera};
use tetra::math::Vec2;
use tetra::input::{self, Key};
use rustbreak::FileDatabase;
use rustbreak::deser::Bincode;
use std::path::Path;
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use serde::{Serialize, Deserialize};
use log::debug;
use std::io::Read;
use rand::rngs::StdRng;
use rand::Rng;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Level {
    width: i32,
    height: i32,
    rooms: Vec<Room>
}
impl Level {
    fn new(w: i32, h: i32) -> Level {
        let mut board = Vec::new();
        for _ in 0..h {
            let row = vec![0; w as usize];
            board.push(row);
        }
        Level {
            width: w,
            height: h,
            rooms: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Room {
    id: i32,
    x: i32,
    y: i32,
    x2: i32,
    y2: i32,
    width: i32,
    height: i32,
    door: (i32, i32),
}

impl Room {
    pub fn new(id: i32, width: i32, height: i32) -> Room {
        let door: (i32, i32) = (0,0);
        Room {
            id,
            x: -(width/2),
            y: -(height/2),
            x2: width/2,
            y2: height/2,
            width,
            height,
            door,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
enum RoomList {
    One,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Door {
    x: f32,
    y: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Player {
    x: f32,
    y: f32,
}
impl Player {
    fn new(_ctx: &mut Context) -> tetra::Result<Player> {
        Ok(Player {
            x: -13.0,
            y: 11.0,
        })
    }
    fn is_up_ok(&self, room: i32) -> bool {
        match room {
            -1 => {
                if self.y >= -67.0 {
                    true
                } else {
                    false
                }
            }
            _=> {
                true
            }
        }
    }
    fn is_down_ok(&self, room: i32) -> bool {
        match room {
            -1 => {
                if self.y <= 105.5 {
                    true
                } else {
                    false
                }
            }
            _=> {
                true
            }
        }
    }
    fn is_left_ok(&self, room: i32) -> bool {
        match room {
            -1 => {
                if self.x >= -122.5 {
                    true
                } else {
                    false
                }
            }
            _=> {
                true
            }
        }
    }
    fn is_right_ok(&self, room: i32) -> bool {
        match room {
            -1 => {
                if self.x <= 106.0 {
                    true
                } else {
                    false
                }
            }
            _ => {
                true
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Environment {
    player: Player,
    room: Room,
}
impl Environment {
    fn new(ctx: &mut Context) -> tetra::Result<Environment> {
        Ok(Environment {
            player: Player::new(ctx)?,
            room: Room::new(-1, -176, -176),
        })
    }
    fn is_door(&self) -> bool {
        if self.player.x > self.room.door.0 as f32 && self.player.x < self.room.door.0 as f32 + 32.0 && self.player.y > self.room.door.1 as f32 - 2.0 && self.player.y < self.room.door.1 as f32 + 16.0 {
            true
        } else {
            false
        }
    }
}
struct GameState {
    current_room: i32,
    tile: Texture,
    door: Texture,
    viewport: Camera,
    viewport_speed: f32,
    player: Texture,
    environment: Environment,
    save_db: FileDatabase<HashMap<String, Environment>, Bincode>
}
impl GameState {
    fn new(ctx: &mut Context) -> tetra::Result<GameState> {
        if !Path::new("saves").exists() {
            let _ = fs::create_dir("saves");
            let _ = File::create("saves/main.db");
        } else if !Path::new("saves/main.db").exists() {
            let _ = File::create("saves/main.db");
        }
        let save_db = FileDatabase::from_file(OpenOptions::new().read(true).write(true).open("saves/main.db").unwrap(), HashMap::new()).unwrap();
        Ok(GameState {
            current_room: 0,
            tile: Texture::new(ctx, "Assets/tile.png")?,
            door: Texture::new(ctx, "Assets/transitions/Door.png")?,
            viewport: Camera::with_window_size(ctx),
            viewport_speed: 0.375,
            player: Texture::new(ctx, "Assets/player.png")?,
            environment: Environment::new(ctx)?,
            save_db

        })
    }

    fn get_room(&mut self, id: i32) -> Option<Room> {
        let mut room = String::new();
        let file = File::open("./environment.ron");
        file.unwrap().read_to_string(&mut room);
        let rooms = ron::de::from_str(room.as_str());
        let level: Level = rooms.expect("Struct Room or Level is Missing a Field");
        let mut room = None;
        for i in 0..level.rooms.len(){
            if i == id as usize {
                room = Some(level.rooms[i].clone())
            }
        }
        room
    }
}
impl State for GameState {
    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
        for key in input::get_keys_down(ctx) {
            match key {
                Key::Left => {
                    if self.environment.player.is_left_ok(self.environment.room.id.clone()) {
                        self.environment.player.x -= 1.5;
                        self.viewport.position.x -= self.viewport_speed;
                    }
                }
                Key::Right => {
                    if self.environment.player.is_right_ok(self.environment.room.id.clone()) {
                        self.environment.player.x += 1.5;
                        self.viewport.position.x += self.viewport_speed;
                    }
                }
                Key::Up => {
                    if self.environment.player.is_up_ok(self.environment.room.id.clone()) {
                        self.viewport.position.y -= self.viewport_speed;
                        self.environment.player.y -= 1.5;
                    }
                }
                Key::Down => {
                    if self.environment.player.is_down_ok(self.environment.room.id.clone()) {
                        self.environment.player.y += 1.5;
                        self.viewport.position.y += self.viewport_speed;
                    }
                }
                Key::T => {
                    if self.environment.is_door() {
                        self.environment.room = self.get_room(self.current_room).expect("Failed to Get the Current Room");
                        self.current_room += 1;
                        println!("DOOR: {}, X: {}, Y: {}", self.current_room, self.environment.room.door.0, self.environment.room.door.1);
                    }
                }
                _ => {}
            }
        }

        self.viewport.update();
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        graphics::clear(ctx, Color::rgb(0.0, 0.0, 0.0));
        for i in (-(self.environment.room.width/2)..self.environment.room.width).step_by(22) {
            for j in (0..self.environment.room.height).step_by(22) {
                draw(ctx, &self.tile, Vec2::new(i as f32, j as f32));

            }
        }
        /*match self.environment.room.id {
            -1 => {
                draw(ctx, &self.background, Vec2::new(-128.0, -128.0));
                draw(ctx, &self.door, Vec2::new(self.environment.room.door.0 as f32, self.environment.room.door.1 as f32));
                draw(ctx, &self.player, Vec2::new(self.environment.player.x, self.environment.player.y));
            }

            _ => {
                draw(ctx, &self.player, Vec2::new(self.environment.player.x, self.environment.player.y));
            }
        }*/

        graphics::set_transform_matrix(ctx, self.viewport.as_matrix());
        Ok(())
    }

    fn event(&mut self, _ctx: &mut Context, event: Event) -> tetra::Result {
        match event {
            Event::KeyPressed {key} => {
               // debug!("{} {}", self.environment.player.x, self.environment.player.y);
            }
            _ => {}
        }
        //debug!("{:?}", event);
        Ok(())
    }
}

fn main() -> tetra::Result {
    env_logger::init();
    ContextBuilder::new("The Courtyard", 704, 704)
        .show_mouse(true)
        .build()?
        .run(GameState::new)

}
