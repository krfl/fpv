use nalgebra::{Point3, UnitQuaternion};
use rand::Rng;

use crate::render::framebuffer::Color;
use crate::world::mesh::{Aabb, Mesh};
use crate::world::primitives;
use crate::world::scene::Scene;

/// Ground is 200x200m centered at origin. Keep objects well inside.
const MAP_HALF: f32 = 85.0;
const SPAWN_CLEAR: f32 = 12.0;
const MIN_OBJECT_SPACING: f32 = 8.0;

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

fn in_bounds(x: f32, z: f32) -> bool {
    x.abs() < MAP_HALF && z.abs() < MAP_HALF
}

fn clear_of_spawn(x: f32, z: f32) -> bool {
    (x * x + z * z).sqrt() > SPAWN_CLEAR
}

/// Check minimum distance from all existing placement centers.
fn far_enough(x: f32, z: f32, placed: &[(f32, f32)]) -> bool {
    for &(px, pz) in placed {
        let dx = x - px;
        let dz = z - pz;
        if (dx * dx + dz * dz).sqrt() < MIN_OBJECT_SPACING {
            return false;
        }
    }
    true
}

/// Generate a random freestyle course.
pub fn random_course() -> Scene {
    let mut rng = rand::rng();
    let mut meshes = Vec::new();
    let mut placed: Vec<(f32, f32)> = Vec::new(); // track placement centers

    // Ground plane
    meshes.push(primitives::ground_plane(200.0, 50));

    // Gate circuit: spread across the map with varied angles
    generate_gate_circuit(&mut rng, &mut meshes, &mut placed);

    // Freestyle zones: 3-5 clusters spread around the map
    let num_clusters = rng.random_range(3..=5);
    for _ in 0..num_clusters {
        for _ in 0..10 {
            // Try up to 10 times to find a valid position
            let angle = rng.random_range(0.0..std::f32::consts::TAU);
            let dist = rng.random_range(25.0..75.0f32);
            let cx = angle.cos() * dist;
            let cz = angle.sin() * dist;
            if in_bounds(cx, cz) && clear_of_spawn(cx, cz) && far_enough(cx, cz, &placed) {
                generate_cluster(&mut rng, cx, cz, &mut meshes);
                placed.push((cx, cz));
                break;
            }
        }
    }

    // Scattered obstacles: fill in gaps
    let num_scattered = rng.random_range(10..=18);
    for _ in 0..num_scattered {
        for _ in 0..10 {
            let x = rng.random_range(-75.0..75.0f32);
            let z = rng.random_range(-75.0..75.0f32);
            if in_bounds(x, z) && clear_of_spawn(x, z) && far_enough(x, z, &placed) {
                generate_scattered(&mut rng, x, z, &mut meshes);
                placed.push((x, z));
                break;
            }
        }
    }

    // Power loop pillars near spawn (always present, gives immediate action)
    let pillar_z = -15.0;
    for i in 0..3 {
        let x = -4.0 + i as f32 * 4.0;
        let h = rng.random_range(6.0..10.0);
        meshes.push(primitives::pillar(
            Point3::new(x, 0.0, pillar_z),
            0.4,
            h,
            random_color(&mut rng),
        ));
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

/// Generate gates spread around the map in a rough circuit, not a straight line.
fn generate_gate_circuit(rng: &mut impl Rng, meshes: &mut Vec<Mesh>, placed: &mut Vec<(f32, f32)>) {
    let num_gates = rng.random_range(8..=12);

    // Place gates in a rough elliptical path around the map
    let radius_x = rng.random_range(35.0..65.0f32);
    let radius_z = rng.random_range(35.0..65.0f32);
    let offset_x = rng.random_range(-10.0..10.0f32);
    let offset_z = rng.random_range(-15.0..0.0f32);

    for i in 0..num_gates {
        let angle = (i as f32 / num_gates as f32) * std::f32::consts::TAU
            + rng.random_range(-0.15..0.15); // slight randomness in angle

        let base_x = offset_x + angle.cos() * radius_x;
        let base_z = offset_z + angle.sin() * radius_z;

        // Add jitter
        let x = (base_x + rng.random_range(-8.0..8.0f32)).clamp(-MAP_HALF + 5.0, MAP_HALF - 5.0);
        let z = (base_z + rng.random_range(-8.0..8.0f32)).clamp(-MAP_HALF + 5.0, MAP_HALF - 5.0);

        if !clear_of_spawn(x, z) {
            continue;
        }

        // Gate faces roughly toward the next gate (tangent to the circle)
        let tangent_angle = angle + std::f32::consts::FRAC_PI_2;
        let rot = tangent_angle.to_degrees() + rng.random_range(-20.0..20.0);

        let y = if rng.random_bool(0.2) {
            rng.random_range(1.0..4.0)
        } else {
            0.0
        };

        let width = rng.random_range(2.5..4.0);
        let height = rng.random_range(2.0..3.5);
        let color = random_color(rng);

        meshes.extend(primitives::gate(Point3::new(x, y, z), rot, width, height, color));
        placed.push((x, z));

        // Support pillars for elevated gates
        if y > 0.3 {
            let hw = width / 2.0 + 0.1;
            let pillar_color = Color::new(80, 80, 80);
            let sin_r = rot.to_radians().sin();
            let cos_r = rot.to_radians().cos();
            meshes.push(primitives::pillar(
                Point3::new(x - cos_r * hw, 0.0, z + sin_r * hw),
                0.15, y, pillar_color,
            ));
            meshes.push(primitives::pillar(
                Point3::new(x + cos_r * hw, 0.0, z - sin_r * hw),
                0.15, y, pillar_color,
            ));
        }
    }
}

fn generate_cluster(rng: &mut impl Rng, cx: f32, cz: f32, meshes: &mut Vec<Mesh>) {
    match rng.random_range(0..5) {
        0 => {
            // Pillar forest — spread out for weaving
            let n = rng.random_range(4..=7);
            for _ in 0..n {
                let x = cx + rng.random_range(-6.0..6.0f32);
                let z = cz + rng.random_range(-6.0..6.0f32);
                if !in_bounds(x, z) { continue; }
                let h = rng.random_range(5.0..14.0);
                let r = rng.random_range(0.3..0.7);
                meshes.push(primitives::pillar(Point3::new(x, 0.0, z), r, h, random_color(rng)));
            }
        }
        1 => {
            // Bando building — multi-story with windows
            let gray = Color::new(90, 90, 90);
            let dark = Color::new(60, 60, 60);
            let rot = rng.random_range(0.0..180.0f32);
            // Ground floor
            meshes.push(primitives::cube(Point3::new(cx, 3.0, cz), 6.0, gray));
            // Second floor
            meshes.push(primitives::cube(Point3::new(cx, 9.0, cz), 6.0, dark));
            // Window gates on two sides
            meshes.extend(primitives::gate(
                Point3::new(cx, 0.0, cz + 3.0), rot, 3.5, 4.0, random_color(rng),
            ));
            meshes.extend(primitives::gate(
                Point3::new(cx + 3.0, 0.0, cz), rot + 90.0, 3.5, 4.0, random_color(rng),
            ));
        }
        2 => {
            // Wall gap — tall wall with a flyable gap at the bottom
            let color = random_color(rng);
            let wall_h = rng.random_range(5.0..9.0);
            let rot = rng.random_range(0.0..180.0);
            meshes.push(primitives::wall(
                Point3::new(cx, 3.5, cz), rot, 12.0, wall_h, 0.3, color,
            ));
            meshes.push(primitives::pillar(Point3::new(cx - 5.5, 0.0, cz), 0.35, 3.5, color));
            meshes.push(primitives::pillar(Point3::new(cx + 5.5, 0.0, cz), 0.35, 3.5, color));
            meshes.extend(primitives::gate(
                Point3::new(cx, 0.0, cz), rot, 3.5, 3.0, random_color(rng),
            ));
        }
        3 => {
            // Twin towers with bridge — split-S territory
            let color = random_color(rng);
            let h = rng.random_range(8.0..16.0);
            let gap = rng.random_range(4.0..7.0);
            meshes.push(primitives::pillar(Point3::new(cx - gap, 0.0, cz), 0.7, h, color));
            meshes.push(primitives::pillar(Point3::new(cx + gap, 0.0, cz), 0.7, h, color));
            meshes.push(primitives::wall(
                Point3::new(cx, h - 2.0, cz), 0.0, gap * 2.0 + 1.0, 1.5, 0.5, color,
            ));
            meshes.extend(primitives::gate(
                Point3::new(cx, 0.0, cz), 0.0, gap * 1.5, 3.0, random_color(rng),
            ));
        }
        4 => {
            // Ramp pair — matty flip playground
            let color = random_color(rng);
            let rot = rng.random_range(0.0..360.0);
            meshes.push(primitives::ramp(
                Point3::new(cx - 4.0, 0.0, cz), rot, 5.0, 7.0, 4.0, color,
            ));
            meshes.push(primitives::ramp(
                Point3::new(cx + 4.0, 0.0, cz), rot + 180.0, 5.0, 7.0, 4.0, color,
            ));
            meshes.extend(primitives::gate(
                Point3::new(cx, 0.0, cz), rot + 90.0, 3.0, 3.0, random_color(rng),
            ));
        }
        _ => unreachable!(),
    }
}

fn generate_scattered(rng: &mut impl Rng, x: f32, z: f32, meshes: &mut Vec<Mesh>) {
    match rng.random_range(0..3) {
        0 => {
            // Tall pillar (power loop target)
            let h = rng.random_range(5.0..12.0);
            let r = rng.random_range(0.3..0.8);
            meshes.push(primitives::pillar(Point3::new(x, 0.0, z), r, h, random_color(rng)));
        }
        1 => {
            // Cube obstacle
            let size = rng.random_range(2.0..5.0);
            meshes.push(primitives::cube(Point3::new(x, size / 2.0, z), size, random_color(rng)));
        }
        2 => {
            // Standalone gate (random orientation)
            let rot = rng.random_range(0.0..360.0);
            let w = rng.random_range(2.5..4.0);
            let h = rng.random_range(2.0..3.5);
            meshes.extend(primitives::gate(Point3::new(x, 0.0, z), rot, w, h, random_color(rng)));
        }
        _ => unreachable!(),
    }
}
