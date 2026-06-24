//! Tessellation: converting a B-rep shell into a triangle mesh for display.
//!
//! The GPU only draws triangles, so before anything reaches the screen we walk
//! each face's boundary loop and break it into triangles. For now faces are
//! assumed planar (true for a cube and every polyhedron), so flat fan
//! triangulation from the first vertex is exact. Curved faces will later need
//! adaptive tessellation that respects a chord tolerance.
//!
//! We emit one set of vertices per face (vertices are *not* shared between
//! faces) and give every vertex its face's normal. This produces the crisp
//! faceted look a cube should have, instead of the smeared normals you'd get
//! from averaging across an edge.

use crate::topology::{FaceId, Shell};
use math::{Point3, Vec3};

/// A renderable triangle mesh: flat `xyz` arrays plus triangle indices, ready
/// to drop into a GPU vertex buffer.
#[derive(Clone, Debug, Default)]
pub struct TriMesh {
    pub positions: Vec<f32>,
    pub normals: Vec<f32>,
    pub indices: Vec<u32>,
}

/// Newell's method: a robust polygon normal that works for any planar (or
/// near-planar) loop, regardless of which vertex you start from. More stable
/// than a single cross product, which degenerates at near-collinear corners.
fn newell_normal(points: &[Point3]) -> Vec3 {
    let mut n = Vec3::ZERO;
    let count = points.len();
    for i in 0..count {
        let cur = points[i];
        let next = points[(i + 1) % count];
        n.x += (cur.y - next.y) * (cur.z + next.z);
        n.y += (cur.z - next.z) * (cur.x + next.x);
        n.z += (cur.x - next.x) * (cur.y + next.y);
    }
    n.normalized()
}

/// Tessellate a whole shell into a single triangle mesh.
pub fn tessellate(shell: &Shell) -> TriMesh {
    let mut mesh = TriMesh::default();

    for f in 0..shell.faces.len() {
        let face = FaceId(f);

        // Gather the face's boundary vertices in loop order.
        let loop_points: Vec<Point3> = shell
            .face_loop(face)
            .iter()
            .map(|&he| shell.vertex(shell.half_edge(he).origin).position)
            .collect();

        if loop_points.len() < 3 {
            continue;
        }

        let normal = newell_normal(&loop_points);

        // Emit this face's vertices, recording the index of the first one.
        let base = (mesh.positions.len() / 3) as u32;
        for p in &loop_points {
            mesh.positions.extend([p.x as f32, p.y as f32, p.z as f32]);
            mesh.normals
                .extend([normal.x as f32, normal.y as f32, normal.z as f32]);
        }

        // Fan triangulate: (0, i, i+1) for i in 1..n-1.
        for i in 1..(loop_points.len() as u32 - 1) {
            mesh.indices.extend([base, base + i, base + i + 1]);
        }
    }

    mesh
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::cube;

    #[test]
    fn cube_tessellates_to_twelve_triangles() {
        let mesh = tessellate(&cube(2.0));
        // 6 faces * 2 triangles * 3 indices.
        assert_eq!(mesh.indices.len(), 36);
        // 6 faces * 4 vertices, no sharing.
        assert_eq!(mesh.positions.len(), 6 * 4 * 3);
    }

    #[test]
    fn euler_cube_matches_polygon_cube() {
        // The operator-built cube should tessellate to the same triangle count
        // as the hand-authored one — a sanity check that it's a real 6-quad box.
        let mesh = tessellate(&crate::primitives::cube_euler(2.0));
        assert_eq!(mesh.indices.len(), 36);
        assert_eq!(mesh.positions.len(), 6 * 4 * 3);
    }

    #[test]
    fn outward_normals_point_away_from_centre() {
        // For a cube centred at the origin, each face normal should point in
        // roughly the same direction as the face centroid (i.e. outward).
        let shell = cube(2.0);
        for f in 0..shell.faces.len() {
            let pts: Vec<_> = shell
                .face_loop(FaceId(f))
                .iter()
                .map(|&he| shell.vertex(shell.half_edge(he).origin).position)
                .collect();
            let normal = newell_normal(&pts);
            let centroid = pts.iter().fold(Vec3::ZERO, |a, &b| a + b) / pts.len() as f64;
            assert!(
                normal.dot(centroid) > 0.0,
                "face {f} normal points inward"
            );
        }
    }
}
