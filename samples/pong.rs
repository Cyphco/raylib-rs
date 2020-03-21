use raylib::prelude::*;
use shipyard::prelude::*;
use structopt::StructOpt;
mod options;

const ARENA_WIDTH: f32 = 480.0;
const ARENA_HEIGHT: f32 = 360.0;

pub trait RectExt: std::borrow::BorrowMut<Rectangle> + std::borrow::Borrow<Rectangle> {
    fn center_at(&self, pos: &Vector2) -> Rectangle {
        let r = self.borrow();
        Rectangle::new(
            pos.x - r.width / 2.0,
            pos.y - r.height / 2.0,
            r.width,
            r.height,
        )
    }
}

impl RectExt for Rectangle {}

/// Components
/// - Position, Velocity, Color, AltColor, Shape, Paddle, Ball, GameCtrl (for time management)

mod components {
    use super::*;
    #[derive(Debug)]
    pub struct Position(pub Vector2);
    #[derive(Debug)]
    pub struct Velocity(pub Vector2);

    #[derive(Debug)]
    pub struct MColor(pub Color);

    #[derive(Debug)]
    pub struct AltColor(pub Color);

    #[derive(Debug)]
    pub enum Shape {
        Rect(Rectangle),
        Circle(f32),
    }

    #[derive(Debug)]
    pub struct Paddle;

    #[derive(Debug)]
    pub struct Ball;

    #[derive(Copy, Clone, Debug, PartialEq)]
    pub enum GameState {
        Reset,
        Playing,
        Paused,
    }

    #[derive(Debug)]
    pub struct GameCtrl;

    #[derive(Debug, Default)]
    pub struct TimeKeeper {
        pub real_time: f32,
        pub game_time: f32,
        pub game_delta_time: f32,
        pub real_delta_time: f32,
    }

    #[derive(Debug)]
    pub struct Window {
        pub w: u32,
        pub h: u32,
    }
}
pub use components::*;

/// Systems
/// DrawSys, MoveSys, InitSys
pub mod systems {
    use super::*;
    #[system(ResetSys)]
    pub fn run(
        mut entities: &mut Entities,
        mut state: Unique<&mut GameState>,
        mut pos: &mut Position,
        mut vel: &mut Velocity,
        mut mcol: &mut MColor,
        mut shape: &mut Shape,
        mut paddle: &mut Paddle,
        mut ball: &mut Ball,
    ) {
        if *state == GameState::Reset {
            *state = GameState::Playing;

            // Add Left paddles
            entities.add_entity(
                (&mut pos, &mut vel, &mut mcol, &mut shape, &mut paddle),
                (
                    Position(vec2(10.0, ARENA_HEIGHT / 2.0)),
                    Velocity(Vector2::zero()),
                    MColor(Color::WHITE),
                    Shape::Rect(Rectangle::new(0.0, 0.0, 10.0, 50.0)),
                    Paddle,
                ),
            );

            // Add Left paddles
            entities.add_entity(
                (&mut pos, &mut vel, &mut mcol, &mut shape, &mut paddle),
                (
                    Position(vec2(ARENA_WIDTH - 10.0, ARENA_HEIGHT / 2.0)),
                    Velocity(Vector2::zero()),
                    MColor(Color::WHITE),
                    Shape::Rect(Rectangle::new(0.0, 0.0, 10.0, 50.0)),
                    Paddle,
                ),
            );

            // ADD
            // Add Ball
            entities.add_entity(
                (&mut pos, &mut vel, &mut mcol, &mut shape, &mut ball),
                (
                    Position(vec2(ARENA_WIDTH / 2.0, ARENA_HEIGHT / 2.0)),
                    Velocity(vec2(5.0, 0.0)),
                    MColor(Color::WHITE),
                    Shape::Circle(5.0),
                    Ball,
                ),
            );
        }
    }

    #[system(MoveSys)]
    pub fn run(time: Unique<&TimeKeeper>, mut pos: &mut Position, vel: &Velocity) {
        (&mut pos, &vel).iter().for_each(|(p, v)| {
            p.0 = p.0 + (v.0 * time.game_delta_time);
        });
    }

    #[system(TimeKeeperSys)]
    pub fn run(
        rl: Unique<NonSend<&RaylibHandle>>,
        mut time: Unique<&mut TimeKeeper>,
        state: Unique<&GameState>,
    ) {
        let dt = rl.get_frame_time();
        if *state != GameState::Paused {
            time.game_time += dt;
            time.game_delta_time = dt;
        } else {
            time.game_delta_time = 0.0;
        }
        time.real_time += dt;
        time.real_delta_time = dt;
    }

    #[system(DrawSys)]
    pub fn run(
        mut rl: Unique<NonSend<&mut RaylibHandle>>,
        thread: Unique<NonSendSync<&RaylibThread>>,
        pos: &Position,
        mcolor: &MColor,
        shape: &Shape,
    ) {
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);
        (&pos, &mcolor, &shape)
            .iter()
            .for_each(|(p, c, shape)| match shape {
                Shape::Circle(rad) => {
                    d.draw_circle(p.0.x as i32, p.0.y as i32, *rad, c.0);
                }
                Shape::Rect(rect) => {
                    let rect = rect.center_at(&p.0);
                    d.draw_rectangle(
                        rect.x as i32,
                        rect.y as i32,
                        rect.width as i32,
                        rect.height as i32,
                        c.0,
                    );
                }
            });
    }
}
pub use systems::*;

fn main() {
    let opt = options::Opt::from_args();
    let (mut rl, thread) = opt.open_window("Pong");
    let (w, h) = (opt.width, opt.height);
    rl.set_target_fps(60);

    let world = World::new();
    world.add_unique_non_send_sync(rl);
    world.add_unique_non_send_sync(thread);

    world.add_unique(TimeKeeper::default());
    world.add_unique(GameState::Reset);

    while !world
        .borrow::<Unique<NonSend<&RaylibHandle>>>()
        .window_should_close()
    {
        world.run_system::<ResetSys>();
        world.run_system::<TimeKeeperSys>();
        world.run_system::<MoveSys>();
        world.run_system::<DrawSys>();
    }
}