//! Euler operators: the atomic, invariant-preserving construction primitives.
//!
//! Every operation here moves the counts `(V, E, F)` by a fixed amount that
//! keeps the Euler–Poincaré relation `V - E + F = 2` intact for a genus-0
//! solid. Because each step is individually valid, *any* shell assembled purely
//! from these operators is topologically valid by construction — we never have
//! to trust a soup of polygons the way [`Shell::from_polygons`] does.
//!
//! We implement the three operators needed to build any simply-connected
//! polyhedron:
//!
//! | op     | meaning                       | ΔV | ΔE | ΔF |
//! |--------|-------------------------------|----|----|----|
//! | `mvfs` | make vertex, face, shell      | +1 |  0 | +1 |
//! | `mev`  | make edge + vertex (a "spur") | +1 | +1 |  0 |
//! | `mef`  | make edge + face (split loop) |  0 | +1 | +1 |
//!
//! Each has an inverse (`kvfs`/`kev`/`kef`); we'll add those when modelling
//! operations need to *remove* topology. For now construction is enough.
//!
//! ## How a solid grows
//!
//! `mev` attaches a new vertex by a "spur" edge whose two half-edges both lie
//! in the *same* face loop (out to the new vertex and straight back). A chain
//! of spurs makes a "caterpillar" loop. `mef` then draws a chord across that
//! loop, splitting it into two faces — and in doing so it turns each spur it
//! encloses into a proper edge shared by two faces. Closing the last loop turns
//! the whole assembly watertight.

use crate::topology::{Face, FaceId, HalfEdge, HalfEdgeId, Shell, Vertex, VertexId};
use math::Point3;

impl Shell {
    fn push_vertex(&mut self, position: Point3, half_edge: Option<HalfEdgeId>) -> VertexId {
        let id = VertexId(self.vertices.len());
        self.vertices.push(Vertex {
            position,
            half_edge,
        });
        id
    }

    fn push_face(&mut self, half_edge: HalfEdgeId) -> FaceId {
        let id = FaceId(self.faces.len());
        self.faces.push(Face { half_edge });
        id
    }

    fn push_half_edge(&mut self, he: HalfEdge) -> HalfEdgeId {
        let id = HalfEdgeId(self.half_edges.len());
        self.half_edges.push(he);
        id
    }

    /// **MVFS** — make vertex, face, shell. Creates a fresh shell seeded with a
    /// single vertex sitting in a single face, represented by one degenerate
    /// self-loop half-edge (origin == destination, no twin yet). This is the
    /// topological "sphere with one point on it": `V=1, E=0, F=1`.
    ///
    /// Returns the new vertex, its face, and the seed half-edge to grow from.
    pub fn mvfs(position: Point3) -> (Shell, VertexId, FaceId, HalfEdgeId) {
        let mut shell = Shell::default();
        let v = shell.push_vertex(position, None);
        // Placeholder face index 0; the half-edge points back at it.
        let he = shell.push_half_edge(HalfEdge {
            origin: v,
            twin: None,
            next: HalfEdgeId(0),
            prev: HalfEdgeId(0),
            face: Some(FaceId(0)),
        });
        let f = shell.push_face(he);
        shell.vertices[v.0].half_edge = Some(he);
        (shell, v, f, he)
    }

    /// **MEV** — make edge + vertex. Grows a new vertex `w` at `position`,
    /// joined to `origin(at)` by a new spur edge inserted into `at`'s loop
    /// immediately *before* `at`. Both half-edges of the new edge stay in the
    /// same face.
    ///
    /// Returns `(w, h_back)` where `h_back` is the half-edge leaving `w` (its
    /// origin is `w`), convenient for chaining another `mev` at `w`.
    pub fn mev(&mut self, at: HalfEdgeId, position: Point3) -> (VertexId, HalfEdgeId) {
        let v = self.half_edges[at.0].origin;
        let face = self.half_edges[at.0].face;

        // The seed self-loop is the only twin-less half-edge; reuse it as the
        // outgoing spur half-edge rather than leaving it dangling.
        if self.half_edges[at.0].twin.is_none() {
            let w = self.push_vertex(position, None);
            let h_back = self.push_half_edge(HalfEdge {
                origin: w,
                twin: Some(at),
                next: at,
                prev: at,
                face,
            });
            let h = &mut self.half_edges[at.0];
            h.twin = Some(h_back);
            h.next = h_back;
            h.prev = h_back;
            self.vertices[w.0].half_edge = Some(h_back);
            return (w, h_back);
        }

        let prev = self.half_edges[at.0].prev;
        let w = self.push_vertex(position, None);

        // h_out: v -> w, h_back: w -> v. Spliced as: prev -> h_out -> h_back -> at
        let h_out = HalfEdgeId(self.half_edges.len());
        let h_back = HalfEdgeId(self.half_edges.len() + 1);
        self.push_half_edge(HalfEdge {
            origin: v,
            twin: Some(h_back),
            next: h_back,
            prev,
            face,
        });
        self.push_half_edge(HalfEdge {
            origin: w,
            twin: Some(h_out),
            next: at,
            prev: h_out,
            face,
        });

        self.half_edges[prev.0].next = h_out;
        self.half_edges[at.0].prev = h_back;
        self.vertices[w.0].half_edge = Some(h_back);
        (w, h_back)
    }

    /// **MEF** — make edge + face. Draws a new edge from `origin(ha)` to
    /// `origin(hb)`, where `ha` and `hb` lie in the same loop. This splits that
    /// loop into two, creating a new face. `ha`'s side keeps a new boundary
    /// edge `a→b`; the chain from `ha` up to `prev(hb)`, closed by `b→a`,
    /// becomes the new face.
    ///
    /// Returns the new face.
    pub fn mef(&mut self, ha: HalfEdgeId, hb: HalfEdgeId) -> FaceId {
        let a = self.half_edges[ha.0].origin;
        let b = self.half_edges[hb.0].origin;
        let old_face = self.half_edges[ha.0].face.expect("mef on a faceless edge");

        let pa = self.half_edges[ha.0].prev;
        let pb = self.half_edges[hb.0].prev;

        // ea: a -> b stays with the old face; eb: b -> a bounds the new face.
        let ea = HalfEdgeId(self.half_edges.len());
        let eb = HalfEdgeId(self.half_edges.len() + 1);
        self.push_half_edge(HalfEdge {
            origin: a,
            twin: Some(eb),
            next: hb,
            prev: pa,
            face: Some(old_face),
        });
        self.push_half_edge(HalfEdge {
            origin: b,
            twin: Some(ea),
            next: ha,
            prev: pb,
            face: None, // set when we assign the new face's loop below
        });

        // Old face loop: ... pa -> ea -> hb ...
        self.half_edges[pa.0].next = ea;
        self.half_edges[hb.0].prev = ea;
        // New face loop: ... pb -> eb -> ha ...
        self.half_edges[pb.0].next = eb;
        self.half_edges[ha.0].prev = eb;

        // The new face owns the loop starting at ha (around through eb).
        let new_face = self.push_face(ha);
        let mut cur = ha;
        loop {
            self.half_edges[cur.0].face = Some(new_face);
            cur = self.half_edges[cur.0].next;
            if cur == ha {
                break;
            }
        }

        // Make sure the old face references a half-edge it still owns.
        self.faces[old_face.0].half_edge = ea;
        new_face
    }

    /// Walk the loop of `face` and return its half-edge whose origin is
    /// `vertex`. Panics if none is found. Only unambiguous once the relevant
    /// loop is simple (each vertex appears once) — true after a loop is closed
    /// by `mef`.
    pub fn find_in_face(&self, face: FaceId, vertex: VertexId) -> HalfEdgeId {
        for he in self.face_loop(face) {
            if self.half_edges[he.0].origin == vertex {
                return he;
            }
        }
        panic!("vertex {vertex:?} not found in face {face:?}");
    }

    /// Validate the structural invariants of the half-edge graph. Returns
    /// `Err` with a description on the first violation. Invaluable for catching
    /// pointer-surgery bugs in the operators above.
    pub fn validate(&self) -> Result<(), String> {
        for (i, he) in self.half_edges.iter().enumerate() {
            let id = HalfEdgeId(i);
            // next/prev are mutual inverses.
            if self.half_edges[he.next.0].prev != id {
                return Err(format!("he {i}: next.prev != self"));
            }
            if self.half_edges[he.prev.0].next != id {
                return Err(format!("he {i}: prev.next != self"));
            }
            // twin is symmetric, and a twin's origin is our destination.
            if let Some(t) = he.twin {
                if self.half_edges[t.0].twin != Some(id) {
                    return Err(format!("he {i}: twin not symmetric"));
                }
                let dest = self.half_edges[he.next.0].origin;
                if self.half_edges[t.0].origin != dest {
                    return Err(format!("he {i}: twin.origin != destination"));
                }
            }
            // every half-edge in a loop shares the same face.
            if he.face != self.half_edges[he.next.0].face {
                return Err(format!("he {i}: face differs across next link"));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(x: f64, y: f64, z: f64) -> Point3 {
        Point3::new(x, y, z)
    }

    #[test]
    fn mvfs_seed_is_valid() {
        let (shell, _v, _f, _h) = Shell::mvfs(p(0.0, 0.0, 0.0));
        assert_eq!(shell.vertices.len(), 1);
        assert_eq!(shell.faces.len(), 1);
        assert_eq!(shell.euler_characteristic(), 2); // 1 - 0 + 1
        assert!(shell.validate().is_ok());
    }

    #[test]
    fn build_cube_via_euler_operators() {
        let s = crate::primitives::cube_euler(2.0);
        assert!(s.validate().is_ok(), "invariant violation: {:?}", s.validate());
        assert_eq!(s.vertices.len(), 8, "vertices");
        assert_eq!(s.faces.len(), 6, "faces");
        assert_eq!(s.half_edges.len(), 24, "half-edges (12 edges * 2)");
        assert!(s.is_closed(), "cube should be watertight");
        assert_eq!(s.euler_characteristic(), 2, "V - E + F");
        for f in 0..s.faces.len() {
            assert_eq!(
                s.face_loop(crate::topology::FaceId(f)).len(),
                4,
                "every cube face is a quad"
            );
        }
    }

    #[test]
    fn build_triangle_solid() {
        // Two triangular faces back-to-back: V=3, E=3, F=2.
        let (mut s, _v0, f0, h0) = Shell::mvfs(p(0.0, 0.0, 0.0));
        let (_v1, h1) = s.mev(h0, p(1.0, 0.0, 0.0));
        let (_v2, h2) = s.mev(h1, p(0.0, 1.0, 0.0));
        // Close the loop: edge from origin(h2)=v2 back to origin(h0-side)=v0.
        let ha = h2;
        let hb = s.find_in_face(f0, _v0);
        s.mef(ha, hb);

        assert!(s.validate().is_ok());
        assert_eq!(s.vertices.len(), 3);
        assert_eq!(s.faces.len(), 2);
        assert_eq!(s.half_edges.len(), 6); // 3 edges * 2
        assert!(s.is_closed());
        assert_eq!(s.euler_characteristic(), 2);
    }
}
