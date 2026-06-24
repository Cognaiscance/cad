import * as THREE from "three";
import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";
import initKernel, { build_cube } from "../kernel-pkg/kernel.js";

/**
 * Entry point. Boots the Rust/WASM geometry kernel, asks it for a cube, and
 * renders the resulting triangle mesh. This is the full pipeline in miniature:
 *
 *   Rust kernel (topology -> tessellate) --wasm--> TypeScript --three.js--> GPU
 */
async function main() {
  // 1. Initialise the WASM module, then call into the kernel.
  await initKernel();
  const mesh = build_cube(2.0);

  // 2. Lift the kernel's typed arrays straight into a GPU-ready geometry.
  const geometry = new THREE.BufferGeometry();
  geometry.setAttribute("position", new THREE.BufferAttribute(mesh.positions, 3));
  geometry.setAttribute("normal", new THREE.BufferAttribute(mesh.normals, 3));
  geometry.setIndex(new THREE.BufferAttribute(mesh.indices, 1));

  // 3. Standard three.js scene scaffolding.
  const canvas = document.getElementById("app") as HTMLCanvasElement;
  const renderer = new THREE.WebGLRenderer({ canvas, antialias: true });
  renderer.setPixelRatio(window.devicePixelRatio);

  const scene = new THREE.Scene();
  scene.background = new THREE.Color(0x1a1d21);

  const camera = new THREE.PerspectiveCamera(50, 1, 0.01, 1000);
  camera.position.set(4, 3, 5);

  const controls = new OrbitControls(camera, canvas);
  controls.enableDamping = true;

  // A solid material plus a couple of lights so the facets read clearly.
  const material = new THREE.MeshStandardMaterial({
    color: 0x4a90d9,
    metalness: 0.1,
    roughness: 0.55,
    flatShading: true,
  });
  scene.add(new THREE.Mesh(geometry, material));

  scene.add(new THREE.HemisphereLight(0xffffff, 0x303338, 0.9));
  const key = new THREE.DirectionalLight(0xffffff, 1.1);
  key.position.set(5, 8, 6);
  scene.add(key);

  // Edge overlay so the cube's topology is visible, not just its faces.
  scene.add(
    new THREE.LineSegments(
      new THREE.EdgesGeometry(geometry, 1),
      new THREE.LineBasicMaterial({ color: 0x0c0f12 })
    )
  );

  scene.add(new THREE.AxesHelper(2.5));

  // 4. Keep the canvas matched to its display size.
  function resize() {
    const w = canvas.clientWidth;
    const h = canvas.clientHeight;
    if (canvas.width !== w || canvas.height !== h) {
      renderer.setSize(w, h, false);
      camera.aspect = w / h;
      camera.updateProjectionMatrix();
    }
  }
  window.addEventListener("resize", resize);

  // 5. Render loop.
  renderer.setAnimationLoop(() => {
    resize();
    controls.update();
    renderer.render(scene, camera);
  });
}

main().catch((err) => {
  console.error(err);
  document.getElementById("hud")!.textContent = "failed to start: " + err;
});
