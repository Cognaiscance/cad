//! Confirms the Euler-operator cube has consistently outward-facing loops, so
//! it renders lit rather than inside-out. (Kept as an integration test since it
//! exercises the crate exactly as the WASM frontend does.)

use kernel::primitives::cube_euler;
use kernel::topology::FaceId;
use math::Vec3;

#[test]
fn euler_cube_normals_point_outward() {
    let s = cube_euler(2.0);
    for f in 0..s.faces.len() {
        let pts: Vec<_> = s
            .face_loop(FaceId(f))
            .iter()
            .map(|&he| s.vertex(s.half_edge(he).origin).position)
            .collect();

        // Newell normal of the face loop.
        let mut n = Vec3::ZERO;
        let c = pts.len();
        for i in 0..c {
            let a = pts[i];
            let b = pts[(i + 1) % c];
            n.x += (a.y - b.y) * (a.z + b.z);
            n.y += (a.z - b.z) * (a.x + b.x);
            n.z += (a.x - b.x) * (a.y + b.y);
        }

        // For a convex solid centred at the origin, an outward normal has a
        // positive dot product with the face centroid.
        let centroid = pts.iter().fold(Vec3::ZERO, |acc, &p| acc + p) / pts.len() as f64;
        let dot = n.dot(centroid);
        assert!(dot > 0.0, "face {f} normal points inward (dot = {dot:.3})");
    }
}
