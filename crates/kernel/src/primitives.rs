//! Construction of basic primitive solids.
//!
//! For now this builds polyhedra directly as polygon soup. As the kernel gains
//! analytic surfaces and sweep operations, primitives like the cylinder and
//! sphere will be built from real geometry rather than facets.

use crate::topology::{Shell, VertexId};
use math::Point3;

/// An axis-aligned cube of the given edge length, centred on the origin.
///
/// Vertices are numbered:
/// ```text
///        7--------6
///       /|       /|
///      4--------5 |
///      | |      | |        y
///      | 3------|-2        |
///      |/       |/         o--- x
///      0--------1         /
///                        z
/// ```
pub fn cube(size: f64) -> Shell {
    let h = size / 2.0;
    let positions: [Point3; 8] = [
        Point3::new(-h, -h, -h), // 0
        Point3::new(h, -h, -h),  // 1
        Point3::new(h, h, -h),   // 2
        Point3::new(-h, h, -h),  // 3
        Point3::new(-h, -h, h),  // 4
        Point3::new(h, -h, h),   // 5
        Point3::new(h, h, h),    // 6
        Point3::new(-h, h, h),   // 7
    ];

    // Each face is wound counter-clockwise as seen from outside, so its outward
    // normal follows the right-hand rule around the loop.
    let faces = vec![
        vec![1, 2, 6, 5], // +X
        vec![0, 4, 7, 3], // -X
        vec![3, 7, 6, 2], // +Y
        vec![0, 1, 5, 4], // -Y
        vec![4, 5, 6, 7], // +Z
        vec![0, 3, 2, 1], // -Z
    ];

    Shell::from_polygons(&positions, &faces)
}

/// The same cube, but constructed entirely through Euler operators instead of
/// trusted polygon soup. Demonstrates that the operator set can assemble a real
/// solid step by step, with every intermediate state topologically valid.
///
/// The recipe is the classic "block" construction: build the bottom square as a
/// caterpillar of spurs and close it, then raise a vertical spur from each
/// bottom corner and stitch the four side walls — the final wall closure leaves
/// the top face behind automatically.
pub fn cube_euler(size: f64) -> Shell {
    let h = size / 2.0;
    let bottom = |x: f64, y: f64| Point3::new(x, y, -h);

    // Bottom square as a chain of spurs: v0 -> v1 -> v2 -> v3.
    let (mut s, v0, f_outer, seed) = Shell::mvfs(bottom(-h, -h));
    let (_v1, e1) = s.mev(seed, bottom(h, -h));
    let (_v2, e2) = s.mev(e1, bottom(h, h));
    let (_v3, e3) = s.mev(e2, bottom(-h, h));

    // Close the bottom: edge from v3 back to v0. `f_outer` keeps the outer loop.
    let back_to_v0 = s.find_in_face(f_outer, v0);
    s.mef(e3, back_to_v0);

    // Raise a vertical spur from each bottom corner, in the outer loop's order,
    // recording the new top vertices in that same order.
    let order: Vec<VertexId> = s
        .face_loop(f_outer)
        .iter()
        .map(|&he| s.half_edge(he).origin)
        .collect();
    let mut tops: Vec<VertexId> = Vec::with_capacity(4);
    for &v in &order {
        let base = s.vertex(v).position;
        let he = s.find_in_face(f_outer, v);
        let (top, _) = s.mev(he, Point3::new(base.x, base.y, h));
        tops.push(top);
    }

    // Stitch the four walls between consecutive top corners. The last `mef`
    // closes the final wall and leaves the top quad as `f_outer`.
    for i in 0..4 {
        let a = s.find_in_face(f_outer, tops[i]);
        let b = s.find_in_face(f_outer, tops[(i + 1) % 4]);
        s.mef(a, b);
    }

    s
}
