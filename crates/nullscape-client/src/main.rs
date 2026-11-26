use bevy::prelude::*;
use nullscape_core::{Action, Simulation, SimulationConfig};
use std::collections::HashMap;

#[derive(Resource)]
struct SimResource(Simulation);

#[derive(Component)]
struct AgentVisual(u32);

#[derive(Component)]
struct EnemyVisual(u32);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window { title: "NullscapeAI — Milestone 1".into(), resolution: (1280., 720.).into(), ..default() }),
            ..default()
        }))
        .insert_resource(SimResource(Simulation::new(SimulationConfig::default())))
        .add_systems(Startup, setup)
        .add_systems(Update, (input_system, sync_visuals))
        .run();
}

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut mats: ResMut<Assets<StandardMaterial>>) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 18.0, 22.0).looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
        ..default()
    });
    commands.spawn(DirectionalLightBundle { transform: Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.8, -0.5, 0.0)), ..default() });

    for id in 0..2 {
        commands.spawn((PbrBundle {
            mesh: meshes.add(Cuboid::new(0.8, 1.8, 0.8)),
            material: mats.add(StandardMaterial { base_color: if id == 0 { Color::srgb(0.2, 0.7, 1.0) } else { Color::srgb(1.0, 0.6, 0.2) }, ..default() }),
            transform: Transform::from_xyz(id as f32 * 3.0, 2.0, 0.0),
            ..default()
        }, AgentVisual(id)));
    }
    commands.spawn((PbrBundle {
        mesh: meshes.add(Cuboid::new(1.2, 1.2, 1.2)),
        material: mats.add(StandardMaterial { base_color: Color::srgb(0.9, 0.15, 0.15), ..default() }),
        transform: Transform::from_xyz(8.0, 1.5, 0.0),
        ..default()
    }, EnemyVisual(0)));
    commands.spawn(PbrBundle {
        mesh: meshes.add(Plane3d::default().mesh().size(64.0, 64.0)),
        material: mats.add(StandardMaterial { base_color: Color::srgb(0.08, 0.12, 0.08), ..default() }),
        ..default()
    });
}

fn input_system(keys: Res<ButtonInput<KeyCode>>, mut sim: ResMut<SimResource>) {
    let mut a = Action::default();
    a.move_z = if keys.pressed(KeyCode::KeyW) { -1.0 } else if keys.pressed(KeyCode::KeyS) { 1.0 } else { 0.0 };
    a.move_x = if keys.pressed(KeyCode::KeyA) { -1.0 } else if keys.pressed(KeyCode::KeyD) { 1.0 } else { 0.0 };
    a.jump = keys.just_pressed(KeyCode::Space);
    a.attack = keys.pressed(KeyCode::KeyF);
    let mut actions = HashMap::new();
    actions.insert(0, a);
    actions.insert(1, Action::default());
    sim.0.step(&actions);
}

fn sync_visuals(
    sim: Res<SimResource>,
    mut agents: Query<(&AgentVisual, &mut Transform)>,
    mut enemies: Query<(&EnemyVisual, &mut Transform)>,
) {
    for (v, mut t) in &mut agents {
        if let Some(a) = sim.0.agents.get(&v.0) { t.translation = a.position.into(); }
    }
    for (v, mut t) in &mut enemies {
        if let Some(e) = sim.0.enemies.iter().find(|e| e.id == v.0) {
            t.translation = e.position;
            t.scale = if e.health > 0.0 { Vec3::ONE } else { Vec3::ZERO };
        }
    }
}
