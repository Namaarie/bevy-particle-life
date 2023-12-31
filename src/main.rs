use bevy::{prelude::*, sprite::MaterialMesh2dBundle, diagnostic::{FrameTimeDiagnosticsPlugin, DiagnosticsStore}, math::vec2, window::PrimaryWindow};
use bevy_egui::{EguiPlugin, egui, EguiContext};
use bevy_inspector_egui::{quick::WorldInspectorPlugin, InspectorOptions, inspector_options::ReflectInspectorOptions};
use rand::Rng;

const NUM_PARTICLES_TYPES: usize = 3;
const NUM_PARTICLES: i32 = 500;
const PARTICLE_SIZE:f32 = 10.0;
const FORCE_MULTIPLIER: f32 = 50.0;
const DISTANCE_MAX: f32 = 100.0;
const FRICTION_HALF_LIFE: f32 = 0.02;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, FrameTimeDiagnosticsPlugin))
        .add_plugins(EguiPlugin)
        .add_plugins(WorldInspectorPlugin::new())
        .init_resource::<RuleSet>()
        .register_type::<RuleSet>()
        .add_systems(Startup, setup)
        .add_systems(Update, ui_system)
        .add_systems(FixedUpdate, apply_forces_between_particles)
        .add_systems(FixedUpdate, apply_movement.after(apply_forces_between_particles))
        .add_systems(Update, ruleset_resource_inspector)
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

#[derive(Resource, Reflect, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
struct RuleSet(
    #[inspector(min = -1.0, max = 1.0)]
    [[f32; NUM_PARTICLES_TYPES]; NUM_PARTICLES_TYPES]
);

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

    fn add_rule_symmetrical(&mut self, type1: ParticleType, type2: ParticleType, value: f32) {
        self.0[usize::from(type1)][usize::from(type2)] = value;
        self.0[usize::from(type2)][usize::from(type1)] = value;
    }
}

impl Default for RuleSet {
    fn default() -> RuleSet {
        let rs = RuleSet([[0.0; NUM_PARTICLES_TYPES]; NUM_PARTICLES_TYPES]);
        rs
    }
}

#[derive(Component)]
struct FPSText;

#[derive(Component, Default)]
struct Velocity(Vec2);

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<ColorMaterial>>, asset_server: Res<AssetServer>, mut ruleset: ResMut<RuleSet>) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn((TextBundle::from_section(
        "",
        TextStyle {
            font_size: 30.,
            ..default()
        }),
        FPSText
    ));

    ruleset.add_rule_symmetrical(ParticleType::BLUE, ParticleType::RED, 1.0);

    ruleset.randomize();

    ruleset.print();

    let mut rng = rand::thread_rng();

    let ball_tex = asset_server.load("circle.png");

    for _i in 1..NUM_PARTICLES {
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
                transform: Transform::from_xyz(rng.gen_range(-400.0..400.0), rng.gen_range(-400.0..400.0), 0.0),
                sprite: Sprite {
                    color: color,
                    custom_size: Some(Vec2 { x: PARTICLE_SIZE, y: PARTICLE_SIZE }),
                    ..default()
                },
                texture: ball_tex.clone(),
                ..default()
            },
            Velocity(vec2(0., 0.)),
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

fn force(r:f32, a:f32) -> f32 {
    let beta = 0.3;
    if r < beta {
        return r / beta - 1.;
    } else if beta < r && r < 1. {
        return a * (1. - f32::abs(2. * r - 1. - beta) / (1. - beta));
    } else {
        return 0.;
    }
}

fn apply_forces_between_particles(
    mut particles: Query<(Entity, &mut Velocity, &mut Transform, &ParticleType)>,
    ruleset: Res<RuleSet>,
    time: Res<Time>
    ) {

    let mut iter = particles.iter_combinations_mut();

    while let Some([(entity, mut velocity, transform, particle_type),
                    (entity_other, _velocity_other, transform_other, particle_type_other)
                    ]) = iter.fetch_next() {
        if entity == entity_other {
            continue;
        }

        let dx = transform_other.translation.x - transform.translation.x;
        let dy = transform_other.translation.y - transform.translation.y;

        let distance = f32::hypot(dx, dy);
        let ruleset_force = ruleset.0[usize::from(*particle_type)][usize::from(*particle_type_other)];

        if distance > 0. && distance < DISTANCE_MAX {
            let force = force(distance / DISTANCE_MAX, ruleset_force);
            
            let mut partial_force = vec2(dx, dy) / distance * force;

            partial_force *= DISTANCE_MAX * FORCE_MULTIPLIER;
            velocity.0 *= f32::powf(0.5, time.delta_seconds() / FRICTION_HALF_LIFE);
            velocity.0 += partial_force * time.delta_seconds();
        }
    }
}

fn apply_movement(mut particles: Query<(&mut Velocity, &mut Transform)>, time: Res<Time>) {
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

fn ruleset_resource_inspector(world: &mut World) {
    let mut egui_context = world
        .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
        .single(world)
        .clone();

        egui::Window::new("Ruleset Inspector")
        .show(egui_context.get_mut(), |ui| {
            bevy_inspector_egui::bevy_inspector::ui_for_resource::<RuleSet>(world, ui);
    });
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
