use nalgebra::{Point3, UnitQuaternion};
use rand::Rng;

use crate::render::framebuffer::Color;
use crate::world::mesh::{Aabb, Mesh};
use crate::world::primitives;
use crate::world::scene::Scene;

/// Ground is 200x200m centered at origin. Keep objects well inside.
const MAP_HALF: f32 = 85.0;
const SPAWN_CLEAR: f32 = 8.0;

const BRIGHT_COLORS: [Color; 8] = [
    Color::new(255, 100, 30),
    Color::new(30, 150, 255),
    Color::new(30, 220, 80),
    Color::new(180, 50, 220),
    Color::new(220, 40, 40),
    Color::new(255, 220, 30),
    Color::new(30, 220, 220),
    Color::new(255, 80, 180),
];

fn random_color(rng: &mut impl Rng) -> Color {
    BRIGHT_COLORS[rng.random_range(0..BRIGHT_COLORS.len())]
}

/// Check if a position is within map bounds.
fn in_bounds(x: f32, z: f32) -> bool {
    x.abs() < MAP_HALF && z.abs() < MAP_HALF
}

/// Check if a position is clear of the spawn area.
fn clear_of_spawn(x: f32, z: f32) -> bool {
    x.abs() > SPAWN_CLEAR || z.abs() > SPAWN_CLEAR
}

/// Generate a random freestyle course.
pub fn random_course() -> Scene {
    let mut rng = rand::rng();
    let mut meshes = Vec::new();

    // Ground plane
    meshes.push(primitives::ground_plane(200.0, 50));

    // Gate path: winding series of gates
    generate_gate_path(&mut rng, &mut meshes);

    // Obstacle clusters
    let num_clusters = rng.random_range(3..=5);
    for _ in 0..num_clusters {
        let cx = rng.random_range(-60.0..60.0f32);
        let cz = rng.random_range(-70.0..50.0f32);
        if !clear_of_spawn(cx, cz) || !in_bounds(cx, cz) {
            continue;
        }
        generate_cluster(&mut rng, cx, cz, &mut meshes);
    }

    // Scattered obstacles
    let num_scattered = rng.random_range(8..=15);
    for _ in 0..num_scattered {
        let x = rng.random_range(-70.0..70.0f32);
        let z = rng.random_range(-70.0..70.0f32);
        if !clear_of_spawn(x, z) || !in_bounds(x, z) {
            continue;
        }
        generate_scattered(&mut rng, x, z, &mut meshes);
    }

    let colliders: Vec<Aabb> = meshes
        .iter()
        .filter(|m| m.collidable)
        .map(|m| m.world_aabb())
        .collect();

    Scene {
        meshes,
        colliders,
        spawn_position: Point3::new(0.0, 0.05, 0.0),
        spawn_orientation: UnitQuaternion::identity(),
    }
}

fn generate_gate_path(rng: &mut impl Rng, meshes: &mut Vec<Mesh>) {
    let num_gates = rng.random_range(8..=12);
    let mut z = -10.0f32;
    let mut x = 0.0f32;

    // Pick a general direction then curve back, staying on the map
    let mut dx_bias = 0.0f32;

    for i in 0..num_gates {
        // Wander laterally but clamp to map bounds
        x += rng.random_range(-5.0..5.0f32) + dx_bias;
        x = x.clamp(-MAP_HALF + 10.0, MAP_HALF - 10.0);
        z -= rng.random_range(6.0..14.0);
        z = z.max(-MAP_HALF + 10.0); // don't go past map edge

        // Bias back toward center in second half
        if i > num_gates / 2 {
            dx_bias = -x * 0.1;
        }

        let y = if rng.random_bool(0.25) {
            rng.random_range(0.5..2.5)
        } else {
            0.0
        };

        let rot = rng.random_range(-30.0..30.0f32);
        let width = rng.random_range(2.5..3.5);
        let height = rng.random_range(2.0..3.0);
        let color = random_color(rng);

        meshes.push(primitives::gate(Point3::new(x, y, z), rot, width, height, color));

        if y > 0.3 {
            let hw = width / 2.0 + 0.1;
            let pillar_color = Color::new(80, 80, 80);
            meshes.push(primitives::pillar(Point3::new(x - hw, 0.0, z), 0.15, y, pillar_color));
            meshes.push(primitives::pillar(Point3::new(x + hw, 0.0, z), 0.15, y, pillar_color));
        }
    }
}

fn generate_cluster(rng: &mut impl Rng, cx: f32, cz: f32, meshes: &mut Vec<Mesh>) {
    match rng.random_range(0..4) {
        0 => {
            // Pillar forest
            let n = rng.random_range(3..=5);
            for _ in 0..n {
                let x = cx + rng.random_range(-3.0..3.0f32);
                let z = cz + rng.random_range(-3.0..3.0f32);
                if !in_bounds(x, z) { continue; }
                let h = rng.random_range(4.0..9.0);
                let r = rng.random_range(0.3..0.6);
                meshes.push(primitives::pillar(Point3::new(x, 0.0, z), r, h, random_color(rng)));
            }
        }
        1 => {
            // Mini bando: cubes with a gate
            let gray = Color::new(90, 90, 90);
            let dark = Color::new(60, 60, 60);
            meshes.push(primitives::cube(Point3::new(cx, 2.5, cz), 5.0, gray));
            meshes.push(primitives::cube(Point3::new(cx, 7.5, cz), 5.0, dark));
            meshes.push(primitives::gate(
                Point3::new(cx, 0.0, cz + 2.5), 0.0, 3.0, 3.5, random_color(rng),
            ));
        }
        2 => {
            // Wall with gap
            let color = random_color(rng);
            let wall_h = rng.random_range(4.0..7.0);
            meshes.push(primitives::wall(
                Point3::new(cx, 2.5, cz), rng.random_range(0.0..180.0), 8.0, wall_h, 0.3, color,
            ));
            meshes.push(primitives::pillar(Point3::new(cx - 3.5, 0.0, cz), 0.3, 2.5, color));
            meshes.push(primitives::pillar(Point3::new(cx + 3.5, 0.0, cz), 0.3, 2.5, color));
            meshes.push(primitives::gate(
                Point3::new(cx, 0.0, cz), 0.0, 3.0, 2.0, random_color(rng),
            ));
        }
        3 => {
            // Twin towers with bridge
            let color = random_color(rng);
            let h = rng.random_range(7.0..12.0);
            meshes.push(primitives::pillar(Point3::new(cx - 3.0, 0.0, cz), 0.6, h, color));
            meshes.push(primitives::pillar(Point3::new(cx + 3.0, 0.0, cz), 0.6, h, color));
            meshes.push(primitives::wall(
                Point3::new(cx, h - 1.5, cz), 0.0, 6.5, 1.0, 0.4, color,
            ));
            meshes.push(primitives::gate(
                Point3::new(cx, 0.0, cz), 0.0, 3.5, 2.5, random_color(rng),
            ));
        }
        _ => unreachable!(),
    }
}

fn generate_scattered(rng: &mut impl Rng, x: f32, z: f32, meshes: &mut Vec<Mesh>) {
    if rng.random_bool(0.5) {
        let h = rng.random_range(3.0..7.0);
        let r = rng.random_range(0.3..0.7);
        meshes.push(primitives::pillar(Point3::new(x, 0.0, z), r, h, random_color(rng)));
    } else {
        let size = rng.random_range(1.5..3.5);
        meshes.push(primitives::cube(Point3::new(x, size / 2.0, z), size, random_color(rng)));
    }
}
