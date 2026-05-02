use nalgebra::{Isometry3, Point3, Vector3};

use crate::render::framebuffer::Color;

#[derive(Clone)]
pub struct Vertex {
    pub position: Point3<f32>,
    pub color: Color,
}

/// Axis-aligned bounding box.
#[derive(Clone, Copy)]
pub struct Aabb {
    pub min: Point3<f32>,
    pub max: Point3<f32>,
}

impl Aabb {
    /// Test if a point is inside the AABB (with padding).
    pub fn contains(&self, point: &Point3<f32>, padding: f32) -> bool {
        point.x >= self.min.x - padding
            && point.x <= self.max.x + padding
            && point.y >= self.min.y - padding
            && point.y <= self.max.y + padding
            && point.z >= self.min.z - padding
            && point.z <= self.max.z + padding
    }

    /// Push a point out of the AABB and return the push direction.
    /// Returns None if the point is not inside.
    pub fn push_out(&self, point: &Point3<f32>, padding: f32) -> Option<Vector3<f32>> {
        if !self.contains(point, padding) {
            return None;
        }

        // Find the axis with the smallest penetration
        let penetrations = [
            (point.x - (self.min.x - padding), Vector3::new(-1.0, 0.0, 0.0)),
            ((self.max.x + padding) - point.x, Vector3::new(1.0, 0.0, 0.0)),
            (point.y - (self.min.y - padding), Vector3::new(0.0, -1.0, 0.0)),
            ((self.max.y + padding) - point.y, Vector3::new(0.0, 1.0, 0.0)),
            (point.z - (self.min.z - padding), Vector3::new(0.0, 0.0, -1.0)),
            ((self.max.z + padding) - point.z, Vector3::new(0.0, 0.0, 1.0)),
        ];

        let (min_pen, push_dir) = penetrations
            .iter()
            .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
            .unwrap();

        Some(*push_dir * *min_pen)
    }
}

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<[u32; 3]>,
    pub transform: Isometry3<f32>,
    /// Bounding sphere in local space: (center, radius)
    pub bounds: (Point3<f32>, f32),
    /// Whether this mesh should be used for collision detection
    pub collidable: bool,
}

impl Mesh {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<[u32; 3]>) -> Self {
        let bounds = compute_bounding_sphere(&vertices);
        Self {
            vertices,
            indices,
            transform: Isometry3::identity(),
            bounds,
            collidable: true,
        }
    }

    pub fn with_transform(mut self, transform: Isometry3<f32>) -> Self {
        self.transform = transform;
        self
    }

    pub fn no_collision(mut self) -> Self {
        self.collidable = false;
        self
    }

    /// Get bounding sphere center in world space.
    pub fn world_bounds_center(&self) -> Point3<f32> {
        self.transform * self.bounds.0
    }

    pub fn bounds_radius(&self) -> f32 {
        self.bounds.1
    }

    /// Compute world-space AABB for collision.
    pub fn world_aabb(&self) -> Aabb {
        let mut min = Point3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY);
        let mut max = Point3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY);

        for v in &self.vertices {
            let wp = self.transform * v.position;
            min.x = min.x.min(wp.x);
            min.y = min.y.min(wp.y);
            min.z = min.z.min(wp.z);
            max.x = max.x.max(wp.x);
            max.y = max.y.max(wp.y);
            max.z = max.z.max(wp.z);
        }

        Aabb { min, max }
    }
}

fn compute_bounding_sphere(vertices: &[Vertex]) -> (Point3<f32>, f32) {
    if vertices.is_empty() {
        return (Point3::origin(), 0.0);
    }

    // Compute centroid
    let mut cx = 0.0f32;
    let mut cy = 0.0f32;
    let mut cz = 0.0f32;
    for v in vertices {
        cx += v.position.x;
        cy += v.position.y;
        cz += v.position.z;
    }
    let n = vertices.len() as f32;
    let center = Point3::new(cx / n, cy / n, cz / n);

    // Compute max distance from centroid
    let mut max_r2 = 0.0f32;
    for v in vertices {
        let d = (v.position - center).norm_squared();
        if d > max_r2 {
            max_r2 = d;
        }
    }

    (center, max_r2.sqrt())
}
