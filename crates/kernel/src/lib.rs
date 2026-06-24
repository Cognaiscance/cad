//! `kernel` — the boundary-representation geometry kernel.
//!
//! This crate is the long-lived core of the CAD system. It knows about
//! geometry and topology and *nothing* about the application on top of it: no
//! UI, no feature history, no undo, no documents. Data flows one way — the app
//! commands the kernel; the kernel never reaches back.
//!
//! Module map (today):
//!   - [`topology`]    — the half-edge connectivity structure (the B-rep shell)
//!   - [`primitives`]  — constructors for basic solids
//!   - [`tessellate`]  — B-rep → triangle mesh for display
//!
//! Everything below the `wasm` boundary is plain Rust and fully unit-testable
//! on native targets. The `#[wasm_bindgen]` layer at the bottom is a thin
//! adapter that exposes a few entry points to the TypeScript frontend.

pub mod euler;
pub mod primitives;
pub mod tessellate;
pub mod topology;

use wasm_bindgen::prelude::*;

/// A triangle mesh handed across the WASM boundary to the frontend.
///
/// The getters return typed arrays (`Float32Array` / `Uint32Array` on the JS
/// side) that can be uploaded straight into a GPU buffer with no per-element
/// copying in JavaScript.
#[wasm_bindgen]
pub struct Mesh {
    inner: tessellate::TriMesh,
}

#[wasm_bindgen]
impl Mesh {
    /// Interleaved `[x, y, z, x, y, z, ...]` vertex positions.
    #[wasm_bindgen(getter)]
    pub fn positions(&self) -> Vec<f32> {
        self.inner.positions.clone()
    }

    /// Per-vertex `[x, y, z, ...]` normals, parallel to `positions`.
    #[wasm_bindgen(getter)]
    pub fn normals(&self) -> Vec<f32> {
        self.inner.normals.clone()
    }

    /// Triangle indices into the position/normal arrays.
    #[wasm_bindgen(getter)]
    pub fn indices(&self) -> Vec<u32> {
        self.inner.indices.clone()
    }
}

/// Build a cube of the given edge length and return its display mesh.
///
/// This is the first end-to-end path through the system: construct topology,
/// tessellate it, and ship triangles to the renderer.
#[wasm_bindgen]
pub fn build_cube(size: f64) -> Mesh {
    let shell = primitives::cube(size);
    Mesh {
        inner: tessellate::tessellate(&shell),
    }
}

/// Build a cube constructed purely through Euler operators (`mvfs`/`mev`/`mef`)
/// and return its display mesh. Geometrically identical to [`build_cube`], but
/// proves the operator-based construction path end to end.
#[wasm_bindgen]
pub fn build_cube_euler(size: f64) -> Mesh {
    let shell = primitives::cube_euler(size);
    Mesh {
        inner: tessellate::tessellate(&shell),
    }
}
