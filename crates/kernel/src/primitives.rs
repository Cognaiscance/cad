//! Construction of basic primitive solids.
//!
//! For now this builds polyhedra directly as polygon soup. As the kernel gains
//! analytic surfaces and sweep operations, primitives like the cylinder and
//! sphere will be built from real geometry rather than facets.

use crate::topology::Shell;
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
