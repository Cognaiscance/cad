import { defineConfig } from "vite";

// The kernel ships as an ESM glue file plus a .wasm binary in ./kernel-pkg,
// produced by `wasm-pack build --target web`. Vite serves and bundles both
// directly; no special plugin is needed for the `--target web` output.
export default defineConfig({
  server: { fs: { allow: [".."] } },
});
