#![allow(unused)]

// mod components;
// mod resources;
// mod setup;

use bevy::input::mouse::{MouseWheel, MouseMotion};
use bevy::input::touchpad::{TouchpadMagnify, TouchpadRotate};
use bevy::prelude::*;
use bevy::winit::WinitSettings;
use bevy::sprite::MaterialMesh2dBundle;


const BACKGROUND_COLOR: Color = Color::rgb(0.2, 0.2, 0.2);
const FRAMERATE_LIMIT: f64 = 30.0;
const POS_FONT_SIZE: f32 = 20.0;
const TEXT_COLOR: Color = Color::rgb(0.5, 0.5, 1.0);
const POS_TEXT_PADDING: Val = Val::Px(5.0);
const GRID_COLOR: Color = Color::rgb(0.8, 0.8, 0.8);

fn main() {
    App::new()
        // Power-saving reactive rendering for applications.
        .insert_resource(WinitSettings::desktop_app())
        .add_plugins((DefaultPlugins, bevy_framepace::FramepacePlugin))
        .insert_resource(CursorPosition::default())
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        // .add_event::<CollisionEvent>()
        // Configure how frequently our gameplay systems are run
        // .insert_resource(FixedTime::new_from_secs(1.0 / 60.0))
        .add_systems(Startup, setup)
        // Add our gameplay simulation systems to the fixed timestep schedule
        // .add_systems(
        //     FixedUpdate,
        //     (
        //         check_for_collisions,
        //         apply_velocity.before(check_for_collisions),
        //         move_paddle
        //             .before(check_for_collisions)
        //             .after(apply_velocity),
        //         play_collision_sound.after(check_for_collisions),
        //     ),
        // )
        .add_systems(Update, (update_cursor_position, touchpad_gestures, pan_zoom_camera))
        .run();

    // .add_systems(Startup, add_people)
    // .add_systems(Update, (hello_world, greet_people))
    // .run();
}


// Add the game's entities to our world
pub fn setup(
    mut commands: Commands,
    mut fs_settings: ResMut<bevy_framepace::FramepaceSettings>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut windows: Query<&mut Window>,
    asset_server: Res<AssetServer>,
) {
    fs_settings.limiter = bevy_framepace::Limiter::from_framerate(FRAMERATE_LIMIT);

    // Camera
    commands.spawn((Camera2dBundle::default(), PanZoomCamera::default()));

    windows.iter_mut().next().unwrap().cursor.icon = CursorIcon::Crosshair;
    // Cursor position
    commands.spawn(
        TextBundle::from_sections([
            TextSection::new(
                "(x, y)", // placeholder
                TextStyle {
                    font_size: POS_FONT_SIZE,
                    color: TEXT_COLOR,
                    ..default()
                },
            ),
            TextSection::new(
                " scale: ",
                TextStyle {
                    font_size: POS_FONT_SIZE,
                    color: TEXT_COLOR,
                    ..default()
                },
            ),
            TextSection::new(
                "1.0", // placeholder
                TextStyle {
                    font_size: POS_FONT_SIZE,
                    color: TEXT_COLOR,
                    ..default()
                },
            ),
        ])
        .with_style(Style {
            position_type: PositionType::Absolute,
            left: POS_TEXT_PADDING,
            bottom: POS_TEXT_PADDING,
            ..default()
        }),
    );

    // TODO: add grid
    commands.spawn(
        (MaterialMesh2dBundle {
            mesh: meshes.add(shape::Circle::default().into()).into(),
            material: materials.add(ColorMaterial::from(GRID_COLOR)),
            transform: Transform::from_translation(Vec3 {
                x: 0.0,
                y: -50.0,
                z: 1.0,
            })
            .with_scale(Vec3 {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            }),
            ..default()
        }),
    );

    // commands.spawn((
    // MaterialMesh2dBundle {
    //     mesh: meshes.add(shape::Circle::default().into()).into(),
    //     material: materials.add(ColorMaterial::from(BALL_COLOR)),
    //     transform: Transform::from_translation(BALL_STARTING_POSITION).with_scale(BALL_SIZE),
    //     ..default()
    // },
    // Ball,
    // Velocity(INITIAL_BALL_DIRECTION.normalize() * BALL_SPEED),
    // ));
}

#[derive(Component)]
pub struct PanZoomCamera {

}

impl Default for PanZoomCamera {
    fn default() -> Self {
        Self {  }
    }
}

/// Location within the grid
#[derive(Resource, Default)]
pub struct CursorPosition(Vec2);

impl CursorPosition {
    fn disp_str(&self) -> String {
        format!("{:.3} mm, {:.3} mm", self.0.x, self.0.y)
    }
}

pub fn update_cursor_position(
    mut pos: ResMut<CursorPosition>,
    mut text_query: Query<&mut Text>,
    window_query: Query<&Window>,
    projection_query: Query<&mut OrthographicProjection, With<PanZoomCamera>>,
) {
    let window = window_query.single();

    if let Some(new_pos) = window.cursor_position() {
        let scale = projection_query.single().scale;
        pos.0 = new_pos * scale;

        let mut text = text_query.single_mut();
        text.sections[0].value = pos.disp_str();
        // Display 1/scale because 
        text.sections[2].value = format!("{:.2}", 1.0 / scale);
    }
}

// these only work on macOS
fn touchpad_gestures(
    mut evr_touchpad_magnify: EventReader<TouchpadMagnify>,
    mut q: Query<&mut OrthographicProjection, With<PanZoomCamera>>, // mut evr_touchpad_rotate: EventReader<TouchpadRotate>,
) {
    let mut projection = q.single_mut();
    for ev_magnify in evr_touchpad_magnify.iter() {
        // Positive zooms in, negative zooms out. All numbers have a pretty
        // small abs.
        projection.scale *= (1.0 + ev_magnify.0);
    }
}

/// Based on https://bevy-cheatbook.github.io/cookbook/pan-orbit-camera.html
fn pan_zoom_camera(
    mut ev_motion: EventReader<MouseMotion>,
    mut ev_scroll: EventReader<MouseWheel>,
    input_mouse: Res<Input<MouseButton>>,
    mut query: Query<(&mut PanZoomCamera, &mut Transform, &Projection)>,
    window_query: Query<&Window>,
) {
    // let orbit_button = MouseButton::Right;
    let pan_button = MouseButton::Middle;

    let mut pan = Vec2::ZERO;
    let mut rotation_move = Vec2::ZERO;
    let mut scroll = 0.0;
    // let mut orbit_button_changed = false;

    
    if input_mouse.pressed(pan_button) {
        for ev in ev_motion.iter() {
            pan += ev.delta;
        }
        dbg!(pan);
    }

    for ev in ev_scroll.iter() {
        scroll += ev.y;
        dbg!(scroll);
    }
    // dbg!(pan, scroll);

    for (mut pan_orbit, mut transform, projection) in query.iter_mut() {
        let mut any = false;
        if pan.length_squared() > 0.0 {
            // any = true;
            // // make panning distance independent of resolution and FOV,
            // let window = get_primary_window_size(&window_query);
            // if let Projection::Perspective(projection) = projection {
            //     pan *= Vec2::new(projection.fov * projection.aspect_ratio, projection.fov) / window;
            // }
            // // translate by local axes
            // let right = transform.rotation * Vec3::X * -pan.x;
            // let up = transform.rotation * Vec3::Y * pan.y;
            // // make panning proportional to distance away from focus point
            // let translation = (right + up) * pan_orbit.radius;
            // pan_orbit.focus += translation;
        } else if scroll.abs() > 0.0 {
            // any = true;
            // pan_orbit.radius -= scroll * pan_orbit.radius * 0.2;
            // // dont allow zoom to reach zero or you get stuck
            // pan_orbit.radius = f32::max(pan_orbit.radius, 0.05);
        }

        if any {
            // // emulating parent/child to make the yaw/y-axis rotation behave like a turntable
            // // parent = x and y rotation
            // // child = z-offset
            // let rot_matrix = Mat3::from_quat(transform.rotation);
            // transform.translation = pan_orbit.focus + rot_matrix.mul_vec3(Vec3::new(0.0, 0.0, pan_orbit.radius));
        }
    }

    // consume any remaining events, so they don't pile up if we don't need them
    // (and also to avoid Bevy warning us about not checking events every frame update)
    ev_motion.clear();
}

fn get_primary_window_size(window_query: &Query<&Window>) -> Vec2 {
    let window = window_query.single();
    Vec2::new(window.width(), window.height() )
}

// // These constants are defined in `Transform` units.
// // Using the default 2D camera they correspond 1:1 with screen pixels.
// const PADDLE_SIZE: Vec3 = Vec3::new(120.0, 20.0, 0.0);
// const GAP_BETWEEN_PADDLE_AND_FLOOR: f32 = 60.0;
// const PADDLE_SPEED: f32 = 500.0;
// // How close can the paddle get to the wall
// const PADDLE_PADDING: f32 = 10.0;

// // We set the z-value of the ball to 1 so it renders on top in the case of overlapping sprites.
// const BALL_STARTING_POSITION: Vec3 = Vec3::new(0.0, -50.0, 1.0);
// const BALL_SIZE: Vec3 = Vec3::new(30.0, 30.0, 0.0);
// const BALL_SPEED: f32 = 400.0;
// const INITIAL_BALL_DIRECTION: Vec2 = Vec2::new(0.5, -0.5);

// const WALL_THICKNESS: f32 = 10.0;
// // x coordinates
// const LEFT_WALL: f32 = -450.;
// const RIGHT_WALL: f32 = 450.;
// // y coordinates
// const BOTTOM_WALL: f32 = -300.;
// const TOP_WALL: f32 = 300.;

// const BRICK_SIZE: Vec2 = Vec2::new(100., 30.);
// // These values are exact
// const GAP_BETWEEN_PADDLE_AND_BRICKS: f32 = 270.0;
// const GAP_BETWEEN_BRICKS: f32 = 5.0;
// // These values are lower bounds, as the number of bricks is computed
// const GAP_BETWEEN_BRICKS_AND_CEILING: f32 = 20.0;
// const GAP_BETWEEN_BRICKS_AND_SIDES: f32 = 20.0;

// const SCOREBOARD_FONT_SIZE: f32 = 40.0;
// const SCOREBOARD_TEXT_PADDING: Val = Val::Px(5.0);

// const BACKGROUND_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);
// const PADDLE_COLOR: Color = Color::rgb(0.3, 0.3, 0.7);
// const BALL_COLOR: Color = Color::rgb(1.0, 0.5, 0.5);
// const BRICK_COLOR: Color = Color::rgb(0.5, 0.5, 1.0);
// const WALL_COLOR: Color = Color::rgb(0.8, 0.8, 0.8);
// const TEXT_COLOR: Color = Color::rgb(0.5, 0.5, 1.0);
// const SCORE_COLOR: Color = Color::rgb(1.0, 0.5, 0.5);

// fn main() {
//     App::new()
//         .add_plugins(DefaultPlugins)
//         .insert_resource(Scoreboard { score: 0 })
//         .insert_resource(ClearColor(BACKGROUND_COLOR))
//         .add_event::<CollisionEvent>()
//         // Configure how frequently our gameplay systems are run
//         .insert_resource(FixedTime::new_from_secs(1.0 / 60.0))
//         .add_systems(Startup, setup)
//         // Add our gameplay simulation systems to the fixed timestep schedule
//         .add_systems(
//             FixedUpdate,
//             (
//                 check_for_collisions,
//                 apply_velocity.before(check_for_collisions),
//                 move_paddle
//                     .before(check_for_collisions)
//                     .after(apply_velocity),
//                 play_collision_sound.after(check_for_collisions),
//             ),
//         )
//         .add_systems(Update, (update_scoreboard, bevy::window::close_on_esc))
//         .run();
// }

// #[derive(Component)]
// struct Paddle;

// #[derive(Component)]
// struct Ball;

// #[derive(Component, Deref, DerefMut)]
// struct Velocity(Vec2);

// #[derive(Component)]
// struct Collider;

// #[derive(Event, Default)]
// struct CollisionEvent;

// #[derive(Component)]
// struct Brick;

// #[derive(Resource)]
// struct CollisionSound(Handle<AudioSource>);

// // This bundle is a collection of the components that define a "wall" in our game
// #[derive(Bundle)]
// struct WallBundle {
//     // You can nest bundles inside of other bundles like this
//     // Allowing you to compose their functionality
//     sprite_bundle: SpriteBundle,
//     collider: Collider,
// }

// /// Which side of the arena is this wall located on?
// enum WallLocation {
//     Left,
//     Right,
//     Bottom,
//     Top,
// }

// impl WallLocation {
//     fn position(&self) -> Vec2 {
//         match self {
//             WallLocation::Left => Vec2::new(LEFT_WALL, 0.),
//             WallLocation::Right => Vec2::new(RIGHT_WALL, 0.),
//             WallLocation::Bottom => Vec2::new(0., BOTTOM_WALL),
//             WallLocation::Top => Vec2::new(0., TOP_WALL),
//         }
//     }

//     fn size(&self) -> Vec2 {
//         let arena_height = TOP_WALL - BOTTOM_WALL;
//         let arena_width = RIGHT_WALL - LEFT_WALL;
//         // Make sure we haven't messed up our constants
//         assert!(arena_height > 0.0);
//         assert!(arena_width > 0.0);

//         match self {
//             WallLocation::Left | WallLocation::Right => {
//                 Vec2::new(WALL_THICKNESS, arena_height + WALL_THICKNESS)
//             }
//             WallLocation::Bottom | WallLocation::Top => {
//                 Vec2::new(arena_width + WALL_THICKNESS, WALL_THICKNESS)
//             }
//         }
//     }
// }

// impl WallBundle {
//     // This "builder method" allows us to reuse logic across our wall entities,
//     // making our code easier to read and less prone to bugs when we change the logic
//     fn new(location: WallLocation) -> WallBundle {
//         WallBundle {
//             sprite_bundle: SpriteBundle {
//                 transform: Transform {
//                     // We need to convert our Vec2 into a Vec3, by giving it a z-coordinate
//                     // This is used to determine the order of our sprites
//                     translation: location.position().extend(0.0),
//                     // The z-scale of 2D objects must always be 1.0,
//                     // or their ordering will be affected in surprising ways.
//                     // See https://github.com/bevyengine/bevy/issues/4149
//                     scale: location.size().extend(1.0),
//                     ..default()
//                 },
//                 sprite: Sprite {
//                     color: WALL_COLOR,
//                     ..default()
//                 },
//                 ..default()
//             },
//             collider: Collider,
//         }
//     }
// }

// // This resource tracks the game's score
// #[derive(Resource)]
// struct Scoreboard {
//     score: usize,
// }

// // Add the game's entities to our world
// fn setup(
//     mut commands: Commands,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut materials: ResMut<Assets<ColorMaterial>>,
//     asset_server: Res<AssetServer>,
// ) {
//     // Camera
//     commands.spawn(Camera2dBundle::default());

//     // Sound
//     let ball_collision_sound = asset_server.load("sounds/breakout_collision.ogg");
//     commands.insert_resource(CollisionSound(ball_collision_sound));

//     // Paddle
//     let paddle_y = BOTTOM_WALL + GAP_BETWEEN_PADDLE_AND_FLOOR;

//     commands.spawn((
//         SpriteBundle {
//             transform: Transform {
//                 translation: Vec3::new(0.0, paddle_y, 0.0),
//                 scale: PADDLE_SIZE,
//                 ..default()
//             },
//             sprite: Sprite {
//                 color: PADDLE_COLOR,
//                 ..default()
//             },
//             ..default()
//         },
//         Paddle,
//         Collider,
//     ));

//     // Ball
//     commands.spawn((
//         MaterialMesh2dBundle {
//             mesh: meshes.add(shape::Circle::default().into()).into(),
//             material: materials.add(ColorMaterial::from(BALL_COLOR)),
//             transform: Transform::from_translation(BALL_STARTING_POSITION).with_scale(BALL_SIZE),
//             ..default()
//         },
//         Ball,
//         Velocity(INITIAL_BALL_DIRECTION.normalize() * BALL_SPEED),
//     ));

//     // Scoreboard
//     commands.spawn(
//         TextBundle::from_sections([
//             TextSection::new(
//                 "Score: ",
//                 TextStyle {
//                     font_size: SCOREBOARD_FONT_SIZE,
//                     color: TEXT_COLOR,
//                     ..default()
//                 },
//             ),
//             TextSection::from_style(TextStyle {
//                 font_size: SCOREBOARD_FONT_SIZE,
//                 color: SCORE_COLOR,
//                 ..default()
//             }),
//         ])
//         .with_style(Style {
//             position_type: PositionType::Absolute,
//             top: SCOREBOARD_TEXT_PADDING,
//             left: SCOREBOARD_TEXT_PADDING,
//             ..default()
//         }),
//     );

//     // Walls
//     commands.spawn(WallBundle::new(WallLocation::Left));
//     commands.spawn(WallBundle::new(WallLocation::Right));
//     commands.spawn(WallBundle::new(WallLocation::Bottom));
//     commands.spawn(WallBundle::new(WallLocation::Top));

//     // Bricks
//     // Negative scales result in flipped sprites / meshes,
//     // which is definitely not what we want here
//     assert!(BRICK_SIZE.x > 0.0);
//     assert!(BRICK_SIZE.y > 0.0);

//     let total_width_of_bricks = (RIGHT_WALL - LEFT_WALL) - 2. * GAP_BETWEEN_BRICKS_AND_SIDES;
//     let bottom_edge_of_bricks = paddle_y + GAP_BETWEEN_PADDLE_AND_BRICKS;
//     let total_height_of_bricks = TOP_WALL - bottom_edge_of_bricks - GAP_BETWEEN_BRICKS_AND_CEILING;

//     assert!(total_width_of_bricks > 0.0);
//     assert!(total_height_of_bricks > 0.0);

//     // Given the space available, compute how many rows and columns of bricks we can fit
//     let n_columns = (total_width_of_bricks / (BRICK_SIZE.x + GAP_BETWEEN_BRICKS)).floor() as usize;
//     let n_rows = (total_height_of_bricks / (BRICK_SIZE.y + GAP_BETWEEN_BRICKS)).floor() as usize;
//     let n_vertical_gaps = n_columns - 1;

//     // Because we need to round the number of columns,
//     // the space on the top and sides of the bricks only captures a lower bound, not an exact value
//     let center_of_bricks = (LEFT_WALL + RIGHT_WALL) / 2.0;
//     let left_edge_of_bricks = center_of_bricks
//         // Space taken up by the bricks
//         - (n_columns as f32 / 2.0 * BRICK_SIZE.x)
//         // Space taken up by the gaps
//         - n_vertical_gaps as f32 / 2.0 * GAP_BETWEEN_BRICKS;

//     // In Bevy, the `translation` of an entity describes the center point,
//     // not its bottom-left corner
//     let offset_x = left_edge_of_bricks + BRICK_SIZE.x / 2.;
//     let offset_y = bottom_edge_of_bricks + BRICK_SIZE.y / 2.;

//     for row in 0..n_rows {
//         for column in 0..n_columns {
//             let brick_position = Vec2::new(
//                 offset_x + column as f32 * (BRICK_SIZE.x + GAP_BETWEEN_BRICKS),
//                 offset_y + row as f32 * (BRICK_SIZE.y + GAP_BETWEEN_BRICKS),
//             );

//             // brick
//             commands.spawn((
//                 SpriteBundle {
//                     sprite: Sprite {
//                         color: BRICK_COLOR,
//                         ..default()
//                     },
//                     transform: Transform {
//                         translation: brick_position.extend(0.0),
//                         scale: Vec3::new(BRICK_SIZE.x, BRICK_SIZE.y, 1.0),
//                         ..default()
//                     },
//                     ..default()
//                 },
//                 Brick,
//                 Collider,
//             ));
//         }
//     }
// }

// fn move_paddle(
//     keyboard_input: Res<Input<KeyCode>>,
//     mut query: Query<&mut Transform, With<Paddle>>,
//     time_step: Res<FixedTime>,
// ) {
//     let mut paddle_transform = query.single_mut();
//     let mut direction = 0.0;

//     if keyboard_input.pressed(KeyCode::Left) {
//         direction -= 1.0;
//     }

//     if keyboard_input.pressed(KeyCode::Right) {
//         direction += 1.0;
//     }

//     // Calculate the new horizontal paddle position based on player input
//     let new_paddle_position =
//         paddle_transform.translation.x + direction * PADDLE_SPEED * time_step.period.as_secs_f32();

//     // Update the paddle position,
//     // making sure it doesn't cause the paddle to leave the arena
//     let left_bound = LEFT_WALL + WALL_THICKNESS / 2.0 + PADDLE_SIZE.x / 2.0 + PADDLE_PADDING;
//     let right_bound = RIGHT_WALL - WALL_THICKNESS / 2.0 - PADDLE_SIZE.x / 2.0 - PADDLE_PADDING;

//     paddle_transform.translation.x = new_paddle_position.clamp(left_bound, right_bound);
// }

// fn apply_velocity(mut query: Query<(&mut Transform, &Velocity)>, time_step: Res<FixedTime>) {
//     for (mut transform, velocity) in &mut query {
//         transform.translation.x += velocity.x * time_step.period.as_secs_f32();
//         transform.translation.y += velocity.y * time_step.period.as_secs_f32();
//     }
// }

// fn update_scoreboard(scoreboard: Res<Scoreboard>, mut query: Query<&mut Text>) {
//     let mut text = query.single_mut();
//     text.sections[1].value = scoreboard.score.to_string();
// }

// fn check_for_collisions(
//     mut commands: Commands,
//     mut scoreboard: ResMut<Scoreboard>,
//     mut ball_query: Query<(&mut Velocity, &Transform), With<Ball>>,
//     collider_query: Query<(Entity, &Transform, Option<&Brick>), With<Collider>>,
//     mut collision_events: EventWriter<CollisionEvent>,
// ) {
//     let (mut ball_velocity, ball_transform) = ball_query.single_mut();
//     let ball_size = ball_transform.scale.truncate();

//     // check collision with walls
//     for (collider_entity, transform, maybe_brick) in &collider_query {
//         let collision = collide(
//             ball_transform.translation,
//             ball_size,
//             transform.translation,
//             transform.scale.truncate(),
//         );
//         if let Some(collision) = collision {
//             // Sends a collision event so that other systems can react to the collision
//             collision_events.send_default();

//             // Bricks should be despawned and increment the scoreboard on collision
//             if maybe_brick.is_some() {
//                 scoreboard.score += 1;
//                 commands.entity(collider_entity).despawn();
//             }

//             // reflect the ball when it collides
//             let mut reflect_x = false;
//             let mut reflect_y = false;

//             // only reflect if the ball's velocity is going in the opposite direction of the
//             // collision
//             match collision {
//                 Collision::Left => reflect_x = ball_velocity.x > 0.0,
//                 Collision::Right => reflect_x = ball_velocity.x < 0.0,
//                 Collision::Top => reflect_y = ball_velocity.y < 0.0,
//                 Collision::Bottom => reflect_y = ball_velocity.y > 0.0,
//                 Collision::Inside => { /* do nothing */ }
//             }

//             // reflect velocity on the x-axis if we hit something on the x-axis
//             if reflect_x {
//                 ball_velocity.x = -ball_velocity.x;
//             }

//             // reflect velocity on the y-axis if we hit something on the y-axis
//             if reflect_y {
//                 ball_velocity.y = -ball_velocity.y;
//             }
//         }
//     }
// }

// fn play_collision_sound(
//     mut commands: Commands,
//     mut collision_events: EventReader<CollisionEvent>,
//     sound: Res<CollisionSound>,
// ) {
//     // Play a sound once per frame if a collision occurred.
//     if !collision_events.is_empty() {
//         // This prevents events staying active on the next frame.
//         collision_events.clear();
//         commands.spawn(AudioBundle {
//             source: sound.0.clone(),
//             // auto-despawn the entity when playback finishes
//             settings: PlaybackSettings::DESPAWN,
//         });
//     }
// }
