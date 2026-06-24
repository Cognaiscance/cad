//! Half-edge topology: the connectivity layer of the boundary representation.
//!
//! This is the "T" in B-rep. It records *how* the boundary is connected —
//! which edges bound which faces, which faces meet at an edge — independent of
//! the actual geometry (the curves and surfaces) that will later be attached.
//!
//! We use the classic **half-edge** structure. Every edge of the solid is split
//! into two oppositely-directed half-edges, one belonging to each of the two
//! faces that share the edge. Each half-edge knows:
//!   - its `origin` vertex,
//!   - the `next` and `prev` half-edges around its face loop,
//!   - its `twin` (the half-edge going the other way on the adjacent face),
//!   - the `face` it bounds.
//!
//! From these links every adjacency query is a constant-time pointer walk.
//!
//! We store everything in `Vec` arenas and refer to elements by integer-handle
//! newtypes rather than Rust references. This is the idiomatic way to build
//! linked graph structures in Rust (it sidesteps the borrow checker for cyclic
//! references) and it gives us stable, cheap, copyable handles — which we will
//! eventually need anyway for the topological-naming problem.

use math::Point3;
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct VertexId(pub usize);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct HalfEdgeId(pub usize);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct FaceId(pub usize);

#[derive(Clone, Debug)]
pub struct Vertex {
    pub position: Point3,
    /// One half-edge that has this vertex as its origin.
    pub half_edge: Option<HalfEdgeId>,
}

#[derive(Clone, Debug)]
pub struct HalfEdge {
    pub origin: VertexId,
    pub twin: Option<HalfEdgeId>,
    pub next: HalfEdgeId,
    pub prev: HalfEdgeId,
    pub face: Option<FaceId>,
}

#[derive(Clone, Debug)]
pub struct Face {
    /// One half-edge on this face's boundary loop.
    pub half_edge: HalfEdgeId,
}

/// A connected boundary surface made of faces, edges, and vertices.
///
/// For a closed solid this is a single watertight shell. The struct does not
/// yet carry the analytic geometry (planes/cylinders/etc.) for its faces — that
/// arrives in a later phase. Today a face's geometry is implied by its vertex
/// loop, which is enough to build and display polyhedra like a cube.
#[derive(Clone, Debug, Default)]
pub struct Shell {
    pub vertices: Vec<Vertex>,
    pub half_edges: Vec<HalfEdge>,
    pub faces: Vec<Face>,
}

impl Shell {
    pub fn vertex(&self, id: VertexId) -> &Vertex {
        &self.vertices[id.0]
    }
    pub fn half_edge(&self, id: HalfEdgeId) -> &HalfEdge {
        &self.half_edges[id.0]
    }
    pub fn face(&self, id: FaceId) -> &Face {
        &self.faces[id.0]
    }

    /// The destination vertex of a half-edge is the origin of its `next`.
    pub fn destination(&self, id: HalfEdgeId) -> VertexId {
        let next = self.half_edge(id).next;
        self.half_edge(next).origin
    }

    /// Walk a face's boundary loop, returning its half-edges in order.
    pub fn face_loop(&self, face: FaceId) -> Vec<HalfEdgeId> {
        let start = self.face(face).half_edge;
        let mut out = vec![start];
        let mut cur = self.half_edge(start).next;
        while cur != start {
            out.push(cur);
            cur = self.half_edge(cur).next;
        }
        out
    }

    /// Build a shell from "polygon soup": a list of vertex positions and a list
    /// of faces, where each face is a loop of vertex indices wound
    /// counter-clockwise as seen from outside the solid.
    ///
    /// This is a convenient way to author simple polyhedra by hand. Later we
    /// will build geometry through Euler operators instead, which guarantee
    /// topological validity step by step; this helper just trusts its input.
    pub fn from_polygons(positions: &[Point3], faces: &[Vec<usize>]) -> Shell {
        let mut shell = Shell {
            vertices: positions
                .iter()
                .map(|&position| Vertex {
                    position,
                    half_edge: None,
                })
                .collect(),
            half_edges: Vec::new(),
            faces: Vec::new(),
        };

        // Maps a directed edge (origin, destination) to its half-edge so we can
        // pair up twins once every face has been laid down.
        let mut directed: HashMap<(usize, usize), HalfEdgeId> = HashMap::new();

        for loop_indices in faces {
            let n = loop_indices.len();
            assert!(n >= 3, "a face needs at least 3 vertices");

            let base = shell.half_edges.len();
            let face_id = FaceId(shell.faces.len());

            // First pass: create the half-edges with origin/face set and
            // next/prev temporarily pointing at themselves.
            for (i, &v) in loop_indices.iter().enumerate() {
                let he_id = HalfEdgeId(base + i);
                shell.half_edges.push(HalfEdge {
                    origin: VertexId(v),
                    twin: None,
                    next: he_id,
                    prev: he_id,
                    face: Some(face_id),
                });
                if shell.vertices[v].half_edge.is_none() {
                    shell.vertices[v].half_edge = Some(he_id);
                }
            }

            // Second pass: stitch the cyclic next/prev links and register each
            // directed edge for twin matching.
            for i in 0..n {
                let cur = base + i;
                let nxt = base + (i + 1) % n;
                shell.half_edges[cur].next = HalfEdgeId(nxt);
                shell.half_edges[nxt].prev = HalfEdgeId(cur);

                let from = loop_indices[i];
                let to = loop_indices[(i + 1) % n];
                let prev = directed.insert((from, to), HalfEdgeId(cur));
                debug_assert!(prev.is_none(), "duplicate directed edge {from}->{to}");
            }

            shell.faces.push(Face {
                half_edge: HalfEdgeId(base),
            });
        }

        // Third pass: pair each directed edge (a,b) with its opposite (b,a).
        for (&(a, b), &he) in &directed {
            if let Some(&twin) = directed.get(&(b, a)) {
                shell.half_edges[he.0].twin = Some(twin);
            }
        }

        shell
    }

    /// Euler–Poincaré check for a closed 2-manifold solid:
    /// `V - E + F = 2 (S - G)`. For a single shell of genus 0 (no through
    /// holes) the right-hand side is 2. A useful sanity check on construction.
    pub fn euler_characteristic(&self) -> i64 {
        let v = self.vertices.len() as i64;
        // Each undirected edge is represented by two half-edges.
        let e = (self.half_edges.len() / 2) as i64;
        let f = self.faces.len() as i64;
        v - e + f
    }

    /// True if every half-edge has a twin — i.e. the shell is closed (no
    /// boundary edges).
    pub fn is_closed(&self) -> bool {
        self.half_edges.iter().all(|he| he.twin.is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::cube;

    #[test]
    fn cube_is_a_valid_closed_solid() {
        let shell = cube(2.0);
        assert_eq!(shell.vertices.len(), 8);
        assert_eq!(shell.faces.len(), 6);
        assert_eq!(shell.half_edges.len(), 24); // 12 edges * 2
        assert!(shell.is_closed());
        // V - E + F = 8 - 12 + 6 = 2  (genus-0 solid)
        assert_eq!(shell.euler_characteristic(), 2);
    }

    #[test]
    fn every_face_loop_has_four_edges() {
        let shell = cube(1.0);
        for f in 0..shell.faces.len() {
            assert_eq!(shell.face_loop(FaceId(f)).len(), 4);
        }
    }
}
