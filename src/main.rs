use bevy::{prelude::*, sprite::MaterialMesh2dBundle, diagnostic::{FrameTimeDiagnosticsPlugin, DiagnosticsStore}, math::vec2};
use rand::Rng;

const NUM_PARTICLES_TYPES: usize = 3;
const NUM_PARTICLES: i32 = 500;
const PARTICLE_SIZE:f32 = 10.0;
const FORCE_MULTIPLIER: f32 = 1.0;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, FrameTimeDiagnosticsPlugin))
        .init_resource::<RuleSet>()
        .add_systems(Startup, setup)
        .add_systems(Update, ui_system)
        .add_systems(FixedUpdate, apply_forces_between_particles)
        .add_systems(FixedUpdate, apply_movement.after(apply_forces_between_particles))
        .run();
}

#[derive(Component, Clone, Copy)]
enum ParticleType {
    RED,
    GREEN,
    BLUE,
}

impl From<ParticleType> for usize {
    fn from(index: ParticleType) -> usize {
        match index {
            ParticleType::RED => 0,
            ParticleType::GREEN => 1,
            ParticleType::BLUE => 2,
        }
    }
}

#[derive(Resource)]
struct RuleSet([[f32; NUM_PARTICLES_TYPES]; NUM_PARTICLES_TYPES]);

impl RuleSet {
    fn print(&self) {
        for row in self.0.iter() {
            for &element in row.iter() {
                print!("{} ", element);
            }
            println!();
        }
    }

    fn randomize(&mut self) {
        let mut rng = rand::thread_rng();
        for row in self.0.iter_mut() {
            for element in row.iter_mut() {
                *element = rng.gen::<f32>() * 2.0 - 1.0;
            }
            println!();
        }
    }
}

impl Default for RuleSet {
    fn default() -> RuleSet {
        let mut rs = RuleSet([[0.0; NUM_PARTICLES_TYPES]; NUM_PARTICLES_TYPES]);
        rs.randomize();
        rs
    }
}

#[derive(Component)]
struct FPSText;

#[derive(Component, Default)]
struct Velocity(Vec2);

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<ColorMaterial>>, asset_server: Res<AssetServer>, ruleset: Res<RuleSet>) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn((TextBundle::from_section(
        "",
        TextStyle {
            font_size: 30.,
            ..default()
        }),
        FPSText
    ));

    ruleset.print();

    let mut rng = rand::thread_rng();

    let ball_tex = asset_server.load("circle.png");

    for i in 1..NUM_PARTICLES {
        let particle_type;
        let color;

        match rng.gen_range(0..=2) {
            0 => particle_type = ParticleType::RED,
            1 => particle_type = ParticleType::GREEN,
            _ => particle_type = ParticleType::BLUE,
        }

        match particle_type {
            ParticleType::BLUE => color = Color::BLUE,
            ParticleType::GREEN => color = Color::GREEN,
            ParticleType::RED => color = Color::RED
        }


        commands.spawn((
            SpriteBundle {
                transform: Transform::from_xyz(i as f32 / NUM_PARTICLES as f32 * 800.0 - 400.0, i as f32 / NUM_PARTICLES as f32 * 800.0 - 400.0, 0.0),
                sprite: Sprite {
                    color: color,
                    custom_size: Some(Vec2 { x: PARTICLE_SIZE, y: PARTICLE_SIZE }),
                    ..default()
                },
                texture: ball_tex.clone(),
                ..default()
            },
            Velocity(vec2(rng.gen::<f32>() * 2.0 - 1.0, rng.gen::<f32>() * 2.0 - 1.0)),
            particle_type
        ));
        
    }

    //border
    commands.spawn(MaterialMesh2dBundle {
        mesh: meshes.add(Mesh::from(shape::Quad::new(Vec2::new(910., 910.)))).into(),
        material: materials.add(ColorMaterial::from(Color::WHITE)),
        transform: Transform::from_xyz(0.0, 0.0, -1.0),
        ..default()
    });

    commands.spawn(MaterialMesh2dBundle {
        mesh: meshes.add(Mesh::from(shape::Quad::new(Vec2::new(900.0, 900.0)))).into(),
        material: materials.add(ColorMaterial::from(Color::rgb_u8(43, 44, 47))),
        transform: Transform::from_xyz(0.0, 0.0, -1.0),
        ..default()
    });
}

fn apply_forces_between_particles(
    mut particles: Query<(Entity, &mut Velocity, &mut Transform, &ParticleType)>,
    ruleset: Res<RuleSet>
    ) {

    let mut iter = particles.iter_combinations_mut();

    while let Some([(entity, mut velocity, transform, particle_type),
                    (entity_other, _velocity_other, transform_other, particle_type_other)
                    ]) = iter.fetch_next() {
        if entity == entity_other {
            continue;
        }

        let mut direction_vector = vec2(transform_other.translation.x - transform.translation.x, transform_other.translation.y - transform.translation.y);

        let distance_squared = direction_vector.length_squared();
        direction_vector = direction_vector.normalize();

        let ruleset_force = ruleset.0[usize::from(*particle_type)][usize::from(*particle_type_other)];

        direction_vector *= ruleset_force * FORCE_MULTIPLIER / distance_squared;

        velocity.0 += direction_vector;
    }
}

fn apply_movement(time: Res<Time>, mut particles: Query<(&mut Velocity, &mut Transform)>) {
    for (mut velocity, mut transform) in &mut particles {
        transform.translation.y += velocity.0.y * time.delta_seconds();

        if transform.translation.y > 450. {
            transform.translation.y = 450.;
            velocity.0.y *= -1.;
        } else if transform.translation.y < -450. {
            transform.translation.y = -450.;
            velocity.0.y *= -1.;
        }

        transform.translation.x += velocity.0.x * time.delta_seconds();

        if transform.translation.x > 450. {
            transform.translation.x = 450.;
            velocity.0.x *= -1.;
        } else if transform.translation.x < -450. {
            transform.translation.x = -450.;
            velocity.0.x *= -1.;
        }
    }
}

fn ui_system(mut query: Query<&mut Text, With<FPSText>>, diag: Res<DiagnosticsStore>) {
    let mut text = query.single_mut();

    let Some(fps) = diag
        .get(FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|fps| fps.smoothed())
    else {
        return;
    };

    text.sections[0].value = format!(
        "FPS: {}",
        fps,
    );
}
