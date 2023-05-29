use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use rand::Rng;

const METRE_WEIGHT: f32 = 1.;
const OFFSET_X: f32 = -300.;
const OFFSET_Y: f32 = -200.;

const METRE: f32 = 100.; // metre to pixel (100 px/m)
const P_NUMW: usize = 10;
const P_NUMH: usize = 30;
const P_SIZE: usize = P_NUMH * P_NUMW;
const P_RADIUS: f32 = METRE / 2.0 / P_NUMW as f32;
const RHO_INIT: f32 = 997.0 / METRE / METRE / METRE;
const P_MASS: f32 = (3 * 3 * 997) as f32 / P_SIZE as f32; // suppose that the length of range z is 3m
const GAS_K: f32 = 138. * METRE * METRE; // unit: m^2/s^2 -> px^2 / s^2
const GRAVITY_ACCEL: f32 = 9.8 * METRE; // 9.8 * 100 px/s^2
const TIME_DELTA: f32 = 0.0005; // 60ms
const H_RADIUS: f32 = 2. * P_RADIUS;
const H_RADIUS_SQ: f32 = H_RADIUS * H_RADIUS;
const WORLD_WIDTH: f32 = 6. * METRE;
const WORLD_HEIGHT: f32 = 4. * METRE;
const GRID_WIDTH: i32 = (WORLD_WIDTH / H_RADIUS) as i32;
const GRID_HEIGHT: i32 = (WORLD_HEIGHT / H_RADIUS) as i32;
const COLLISION_DAMP: f32 = 0.5;

const HRAD_9: f32 = H_RADIUS_SQ * H_RADIUS_SQ * H_RADIUS_SQ * H_RADIUS_SQ * H_RADIUS;
const POLY6_W: f32 = 315. / 64. / std::f32::consts::PI / HRAD_9;
const POLY6_WDEL: f32 = -945. / 32. / std::f32::consts::PI / HRAD_9;
const WALL_COLOR: Color = Color::rgb(0.8, 0.8, 0.8);

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
        .add_systems(
            (
                calculate_map,
                calculate_rho.after(calculate_map),
                apply_rho.after(calculate_rho),
                calculate_accel.after(apply_rho),
                apply_accel.after(calculate_accel),
                update_time.after(apply_accel),
            )
                .in_schedule(CoreSchedule::FixedUpdate),
        )
        .insert_resource(FixedTime::new_from_secs(TIME_DELTA))
        .add_system(render)
        .add_system(update_text)
        .run();
}

pub struct ParticleValue {
    pos: Vec2,
    vel: Vec2,
    rho: f32,
    press: f32,
}

#[derive(Component)]
pub struct ParticleIdx {
    idx: usize,
    map_idx: usize,
    near_particles: Vec<usize>,
    near_dist: Vec<f32>,
    rho: f32,
    press: f32,
    accel: Vec2,
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
    pub fn new() -> Self {
        let mut particles = Vec::with_capacity(P_SIZE);
        let mut particle_map: Vec<Vec<usize>> =
            Vec::with_capacity((GRID_WIDTH * GRID_HEIGHT + 1) as usize);
        let mut rng = rand::thread_rng();

        for ui in 1..=P_NUMH {
            for uj in 1..=P_NUMW {
                let pos = Vec2::new(
                    uj as f32 * (METRE / P_NUMW as f32) - P_RADIUS / 2. + rng.gen_range(-0.1..0.1),
                    ui as f32 * (3. * METRE / P_NUMH as f32) - P_RADIUS / 2.
                        + rng.gen_range(-0.1..0.1),
                );
                particles.push(ParticleValue {
                    pos,
                    vel: Vec2::new(0., 0.),
                    rho: 0.,
                    press: 0.,
                });
            }
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

    let ps = ParticleSystem::new();

    for ui in 1..=P_NUMH {
        for uj in 1..=P_NUMW {
            let idx = (ui - 1) * P_NUMW + uj - 1;

            commands.spawn((
                MaterialMesh2dBundle {
                    mesh: meshes
                        .add(shape::Circle::new(P_RADIUS * METRE_WEIGHT).into())
                        .into(),
                    material: materials.add(
                        Color::rgb(
                            idx as f32 / P_SIZE as f32,
                            0.1,
                            (P_SIZE - idx) as f32 / P_SIZE as f32,
                        )
                        .into(),
                    ),
                    transform: Transform::from_translation(Vec3::new(0., 0., 0.)),
                    ..Default::default()
                },
                ParticleIdx {
                    idx,
                    map_idx: 0,
                    near_particles: Vec::with_capacity(8),
                    near_dist: Vec::with_capacity(8),
                    rho: RHO_INIT,
                    press: 0.,
                    accel: Vec2::new(0., 0.),
                },
            ));
        }
    }

    commands.spawn(ps);

    // Walls
    commands.spawn(SpriteBundle {
        transform: Transform {
            translation: Vec3::new(OFFSET_X, 0., 0.),
            scale: Vec3::new(1., WORLD_HEIGHT, 1.),
            ..default()
        },
        sprite: Sprite {
            color: WALL_COLOR,
            ..default()
        },
        ..default()
    });
    commands.spawn(SpriteBundle {
        transform: Transform {
            translation: Vec3::new(0., OFFSET_Y, 0.),
            scale: Vec3::new(WORLD_WIDTH, 1., 1.),
            ..default()
        },
        sprite: Sprite {
            color: WALL_COLOR,
            ..default()
        },
        ..default()
    });
    commands.spawn(SpriteBundle {
        transform: Transform {
            translation: Vec3::new(-OFFSET_X, 0., 0.),
            scale: Vec3::new(1., WORLD_HEIGHT, 1.),
            ..default()
        },
        sprite: Sprite {
            color: WALL_COLOR,
            ..default()
        },
        ..default()
    });
    commands.spawn(SpriteBundle {
        transform: Transform {
            translation: Vec3::new(0., -OFFSET_Y, 0.),
            scale: Vec3::new(WORLD_WIDTH, 1., 1.),
            ..default()
        },
        sprite: Sprite {
            color: WALL_COLOR,
            ..default()
        },
        ..default()
    });

    // Debug text
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

fn calculate_map(mut ps: Query<&mut ParticleSystem>, mut particle_idx: Query<&mut ParticleIdx>) {
    let mut psa = ps.get_single_mut().unwrap();

    for pmap in &mut psa.particle_map {
        pmap.clear();
    }

    for mut part_idx in &mut particle_idx {
        let i = part_idx.idx;
        let part = &psa.particles[i];

        let mut x = (part.pos.x / H_RADIUS).floor() as i32;
        let mut y = (part.pos.y / H_RADIUS).floor() as i32;

        if x < 0 {
            x = 0;
        } else if x >= GRID_WIDTH {
            x = GRID_WIDTH - 1;
        }

        if y < 0 {
            y = 0;
        } else if y >= GRID_HEIGHT {
            y = GRID_HEIGHT - 1;
        }

        let idx = (y * GRID_WIDTH + x) as usize;
        psa.particle_map[idx].push(i);
        part_idx.map_idx = idx;
    }
}

fn near_grid(idx: i32, near_idx: &mut Vec<i32>) {
    near_idx.clear();

    let x = idx % GRID_WIDTH;
    let y = idx / GRID_WIDTH;

    for i in -1..=1 {
        for j in -1..=1 {
            let nx = x + i;
            let ny = y + j;
            if 0 <= nx && nx < GRID_WIDTH && 0 <= ny && ny < GRID_HEIGHT {
                near_idx.push(ny * GRID_WIDTH + nx);
            }
        }
    }
}

fn calculate_rho(ps: Query<&ParticleSystem>, mut particle_idx: Query<&mut ParticleIdx>) {
    let psa = ps.get_single().unwrap();
    let mut near_idx: Vec<i32> = Vec::new();

    for mut idx in &mut particle_idx {
        idx.near_particles.clear();
        idx.near_dist.clear();

        let curr_part = &psa.particles[idx.idx];
        let pos = curr_part.pos;
        let map_idx = idx.map_idx;

        near_grid(map_idx as i32, &mut near_idx);

        for &map_idx in near_idx.iter() {
            for &part_idx in psa.particle_map[map_idx as usize].iter() {
                let part = &psa.particles[part_idx];
                let dist_sq = (part.pos - pos).length_squared();
                if dist_sq < H_RADIUS_SQ {
                    idx.near_particles.push(part_idx);
                    idx.near_dist.push(dist_sq);
                }
            }
        }

        let mut rho = 0.000001; // div-by-zero protection
        for &near_dist_sq in idx.near_dist.iter() {
            rho += P_MASS * POLY6_W * (H_RADIUS_SQ - near_dist_sq).powi(3);
        }
        idx.rho = rho;
        idx.press = GAS_K * (rho - RHO_INIT);
    }
}

fn apply_rho(mut ps: Query<&mut ParticleSystem>, particle_idx: Query<&ParticleIdx>) {
    let mut psa = ps.get_single_mut().unwrap();
    for idx in &particle_idx {
        psa.particles[idx.idx].rho = idx.rho;
        psa.particles[idx.idx].press = idx.press;
    }
}

fn calculate_accel(ps: Query<&ParticleSystem>, mut particle_idx: Query<&mut ParticleIdx>) {
    let psa = ps.get_single().unwrap();

    for mut idx in &mut particle_idx {
        idx.accel.x = 0.;
        idx.accel.y = -GRAVITY_ACCEL;

        let curr_rho = psa.particles[idx.idx].rho;
        let curr_press = psa.particles[idx.idx].press;
        let curr_pos = psa.particles[idx.idx].pos;
        let coeff = -P_MASS / curr_rho;

        let mut press = Vec2::new(0., 0.);
        for i in 0..idx.near_particles.len() {
            let near_idx = idx.near_particles[i];
            let near_pos = psa.particles[near_idx].pos;
            let near_rho = psa.particles[near_idx].rho;
            let near_press = psa.particles[near_idx].press;
            let dist_sq = idx.near_dist[i];

            let near_coeff = (near_press + curr_press) / (2. * near_rho);

            press +=
                near_coeff * POLY6_WDEL * (H_RADIUS_SQ - dist_sq).powi(2) * (curr_pos - near_pos);
        }

        idx.accel += press * coeff;
    }
}

fn apply_accel(mut ps: Query<&mut ParticleSystem>, particle_idx: Query<&ParticleIdx>) {
    for mut ps in &mut ps {
        for idx in &particle_idx {
            let part = &mut ps.particles[idx.idx];
            part.vel += idx.accel * TIME_DELTA;

            // constraint in window
            let mut next_pos = part.pos + part.vel * TIME_DELTA;
            if next_pos.x < 0. {
                // next_pos.x = -next_pos.x;
                next_pos.x = -next_pos.x + 0.05;
                part.vel.x = -COLLISION_DAMP * part.vel.x;
            } else if next_pos.x > WORLD_WIDTH {
                next_pos.x = WORLD_WIDTH - (next_pos.x - WORLD_WIDTH) - 0.05;
                part.vel.x = -COLLISION_DAMP * part.vel.x
            }

            if next_pos.y < 0. {
                next_pos.y = -next_pos.y + 0.05;
                part.vel.y = -COLLISION_DAMP * part.vel.y;
            } else if next_pos.y > WORLD_HEIGHT {
                next_pos.y = WORLD_HEIGHT - (next_pos.y - WORLD_HEIGHT) - 0.05;
                // part.vel.y = -COLLISION_DAMP * part.vel.y;
                part.vel.y = -COLLISION_DAMP * part.vel.y;
            }

            part.pos = next_pos;
            // if (idx.idx == 0) {
            //     println!("pos {} / vel {} / accel {}", part.pos, part.vel, idx.accel);
            // }
        }
    }
}

fn render(mut particle_pos: Query<(&mut Transform, &mut ParticleIdx)>, ps: Query<&ParticleSystem>) {
    let psa = ps.get_single().unwrap();

    for (mut transform, idx) in &mut particle_pos {
        let part = &psa.particles[idx.idx];
        transform.translation.x = part.pos.x * METRE_WEIGHT + OFFSET_X;
        transform.translation.y = part.pos.y * METRE_WEIGHT + OFFSET_Y;

        if part.pos.x.is_nan() {
            println!("idx {} / pos {}", idx.idx, part.pos);
        }
    }
}

fn update_text(time_state: Res<TimeState>, mut text_query: Query<&mut Text>) {
    let mut text = text_query.single_mut();
    text.sections[1].value = format!("{:0.3}", time_state.elapsed_time);
    text.sections[3].value = time_state.frame.to_string();
    text.sections[5].value = format!("{:0.3}", time_state.fps)
}

fn update_time(mut time_state: ResMut<TimeState>, time: Res<Time>) {
    time_state.frame += 1;
    time_state.elapsed_time = time_state.frame as f32 * TIME_DELTA;
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
