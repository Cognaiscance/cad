# cad

A browser-based parametric CAD system, built from the geometry kernel up.

The long-term goal is an Onshape-style modeller running in the browser, on top
of a **from-scratch boundary-representation (B-rep) geometry kernel written in
Rust** and compiled to WebAssembly.

## Architecture

```
┌──────────────────────────────────────────────┐
│  UI            (TypeScript · three.js)         │  viewport, tools  ← web/
├──────────────────────────────────────────────┤
│  Feature/document engine, sketch solver        │  (future)
├──────────────────────────────────────────────┤  ← WASM boundary
│  GEOMETRY KERNEL  (Rust)                       │  ← crates/kernel
│     └ math primitives                          │  ← crates/math
└──────────────────────────────────────────────┘
```

The kernel knows only geometry and topology — no UI, no history, no documents.
Data flows one way: the app commands the kernel; the kernel never reaches back.
The kernel lives in its own crate with zero app dependencies so it can be split
into a standalone repo/package once it stabilises.

## Layout

| Path            | What it is                                              |
| --------------- | ------------------------------------------------------- |
| `crates/math`   | Geometric math primitives (vectors today; more later).  |
| `crates/kernel` | The B-rep kernel: topology, primitives, tessellation.   |
| `web/`          | Vite + TypeScript + three.js frontend.                  |
| `web/kernel-pkg`| Generated WASM build of the kernel (git-ignored).       |

## Roadmap

- [x] **Phase 0** — workspace, math core, tolerance philosophy (in progress)
- [x] **Phase 1** — half-edge topology + a hand-built cube
- [x] **Phase 3 (early)** — tessellation + three.js viewport (end-to-end!)
- [ ] **Phase 1 cont.** — Euler operators (MEV/MEF) for valid construction
- [ ] **Phase 2** — analytic curves & surfaces (line, circle; plane, cylinder…)
- [ ] **Phase 4** — extrude & revolve a 2D profile into a solid
- [ ] **Phase 5/6** — analytic intersections → boolean operations
- [ ] **Phase 7** — fillets & chamfers
- [ ] **Phase 8** — NURBS & general surface/surface intersection

## Developing

Prerequisites: Rust (with `wasm32-unknown-unknown` target), `wasm-pack`, Node.

```sh
# Build the kernel to WASM (outputs web/kernel-pkg/)
wasm-pack build crates/kernel --target web --out-dir ../../web/kernel-pkg --out-name kernel

# Run the native kernel tests
cargo test

# Run the web app
cd web && npm install && npm run dev
```
