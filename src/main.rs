use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use rand::Rng;

const METRE: f32 = 100.;
const P_NUMW: usize = 10;
const P_NUMH: usize = 30;
const P_SIZE: usize = P_NUMH * P_NUMW;
const P_RADIUS: f32 = METRE / 2.0 / P_NUMW as f32;
const P_MASS: f32 = (3 * 3 * 997) as f32 / P_SIZE as f32; // suppose that the length of range z is 3m
const GAS_K: f32 = 140.0;
const GRAVITY_ACCEL: f32 = 9.8;
const TIME_DELTA: f32 = 0.03; // 30ms
const H_RADIUS: f32 = 3. * P_RADIUS;
const RHO_INIT: f32 = 997.0;
const WORLD_WIDTH: f32 = 6. * METRE;
const WORLD_HEIGHT: f32 = 4. * METRE;
const GRID_WIDTH: usize = WORLD_WIDTH as usize / H_RADIUS as usize;
const GRID_HEIGHT: usize = WORLD_HEIGHT as usize / H_RADIUS as usize;

pub mod hello;

fn main() {
    App::new()
        .insert_resource(TimeState {
            elapsed_time: 0.0,
            frame: 0,
            fps: 0.,
        })
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system(init_particle)
        .add_systems((
            calculate_map,
            calculate_accel.before(apply_accel).after(calculate_map),
            apply_accel.after(calculate_accel),
            render.after(apply_accel),
            update_time.after(render),
        ))
        .add_system(update_text)
        .run();
}

pub struct ParticleValue {
    pos: Vec2,
    vel: Vec2,
    accel: Vec2,
}

#[derive(Component)]
pub struct ParticleIdx {
    idx: usize,
    rho: f32,
}

#[derive(Component)]
pub struct ParticleSystem {
    particles: Vec<ParticleValue>,
    particle_map: Vec<Vec<usize>>,
}

#[derive(Resource)]
struct TimeState {
    elapsed_time: f32,
    frame: i32,
    fps: f32,
}

impl ParticleSystem {
    pub fn new(size: usize) -> Self {
        let mut particles = Vec::with_capacity(size);
        let mut particle_map: Vec<Vec<usize>> = Vec::with_capacity(GRID_WIDTH * GRID_HEIGHT + 1);

        for _ in 0..size {
            particles.push(ParticleValue {
                pos: Vec2::new(0., 0.),
                vel: Vec2::new(0., -1.),
                accel: Vec2::new(0., -0.5),
            });
        }

        for _ in 0..particle_map.capacity() {
            particle_map.push(Vec::with_capacity(8));
        }

        Self {
            particles,
            particle_map,
        }
    }
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(Camera2dBundle::default());

    let mut ps = ParticleSystem::new(P_SIZE);

    for ui in 1..=P_NUMH {
        for uj in 1..=P_NUMW {
            let idx = (ui - 1) * P_NUMW + uj - 1;
            let pos = Vec3::new(
                uj as f32 * (METRE / P_NUMW as f32) - P_RADIUS / 2.,
                ui as f32 * (3. * METRE / P_NUMH as f32) - P_RADIUS / 2.,
                0.,
            );
            ps.particles[idx].pos.x = pos.x;
            ps.particles[idx].pos.y = pos.y;

            commands.spawn((
                MaterialMesh2dBundle {
                    mesh: meshes.add(shape::Circle::new(P_RADIUS).into()).into(),
                    material: materials.add(
                        Color::rgb(
                            idx as f32 / P_SIZE as f32,
                            0.1,
                            (P_SIZE - idx) as f32 / P_SIZE as f32,
                        )
                        .into(),
                    ),
                    transform: Transform::from_translation(pos),
                    ..Default::default()
                },
                ParticleIdx { idx, rho: RHO_INIT },
            ));
        }
    }

    commands.spawn(ps);

    commands.spawn(
        TextBundle::from_sections([
            TextSection::new(
                "Time: ",
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 20.,
                    color: Color::rgb(0.5, 0.5, 1.0),
                },
            ),
            TextSection::from_style(TextStyle {
                font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                font_size: 20.,
                color: Color::rgb(0.5, 0.5, 1.0),
            }),
            TextSection::new(
                ", Frame: ",
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 20.,
                    color: Color::rgb(0.5, 0.5, 1.0),
                },
            ),
            TextSection::from_style(TextStyle {
                font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                font_size: 20.,
                color: Color::rgb(0.5, 0.5, 1.0),
            }),
            TextSection::new(
                ", FPS: ",
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 20.,
                    color: Color::rgb(0.5, 0.5, 1.0),
                },
            ),
            TextSection::from_style(TextStyle {
                font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                font_size: 20.,
                color: Color::rgb(0.5, 0.5, 1.0),
            }),
        ])
        .with_style(Style {
            position_type: PositionType::Absolute,
            position: UiRect {
                top: Val::Px(5.0),
                left: Val::Px(5.0),
                ..default()
            },
            ..default()
        }),
    );
}

fn init_particle(
    time: Res<Time>,
    // record_time: Res<TimeState>,
    mut text_query: Query<&mut Text>,
    mut particle_pos: Query<(&mut Transform, &mut ParticleIdx)>,
    ps: Query<&ParticleSystem>,
) {
    let psa = ps.get_single().unwrap();

    for (mut transform, idx) in &mut particle_pos {
        if idx.idx == 400 {
            transform.translation.x = 100. * time.elapsed_seconds().sin();
            transform.translation.y = 100. * time.elapsed_seconds().cos();
            let mut text = text_query.single_mut();
            text.sections[1].value = format!("{:0.3}", transform.translation.x);
            text.sections[3].value = format!("{:0.3}", transform.translation.y);
        }
    }

    // for (mut transform, idx) in &mut particle_pos {
    //     let rho = psa.particles[idx.idx].rho;
    //     transform.translation.x = 100. * rho * time.elapsed_seconds().sin();
    //     transform.translation.y = idx.idx as f32;
    // }
}

fn calculate_accel() {}

fn calculate_map() {}

fn apply_accel(mut ps: Query<&mut ParticleSystem>) {
    for mut ps in &mut ps {
        for i in 0..ps.particles.len() {
            let part = &mut ps.particles[i];
            part.vel += part.accel * TIME_DELTA;
            part.pos += part.vel * TIME_DELTA;
        }
    }
}

fn render(mut particle_pos: Query<(&mut Transform, &mut ParticleIdx)>, ps: Query<&ParticleSystem>) {
    let psa = ps.get_single().unwrap();

    for (mut transform, idx) in &mut particle_pos {
        transform.translation.x = psa.particles[idx.idx].pos.x;
        transform.translation.y = psa.particles[idx.idx].pos.y;
    }
}

fn update_text(time_state: Res<TimeState>, mut text_query: Query<&mut Text>) {
    let mut text = text_query.single_mut();
    text.sections[1].value = format!("{:0.3}", time_state.elapsed_time);
    text.sections[3].value = time_state.frame.to_string();
    text.sections[5].value = format!("{:0.3}", time_state.fps)
}

fn update_time(mut time_state: ResMut<TimeState>, time: Res<Time>) {
    time_state.elapsed_time = time.elapsed_seconds();
    time_state.frame += 1;
    time_state.fps = 1. / time.delta_seconds();
}

// fn update_particle(
//     time: Res<Time>,
//     mut particle_pos: Query<(&mut Transform, &mut ParticleIdx)>,
//     ps: Query<&ParticleSystem>,
// ) {
// }

// The sprite is animated by changing its translation depending on the time that has passed since
// the last frame.
// fn sprite_movement(time: Res<Time>, mut sprite_position: Query<(&mut Direction, &mut Transform)>) {
//     for (mut logo, mut transform) in &mut sprite_position {
//         match *logo {
//             Direction::Up => transform.translation.y += 150. * time.delta_seconds(),
//             Direction::Down => transform.translation.y -= 150. * time.delta_seconds(),
//         }

//         if transform.translation.y > 200. {
//             *logo = Direction::Down;
//         } else if transform.translation.y < -200. {
//             *logo = Direction::Up;
//         }
//     }
// }
