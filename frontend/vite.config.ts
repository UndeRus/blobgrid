import { sveltekit } from "@sveltejs/kit/vite";
import { defineConfig } from "vite";

export default defineConfig({
  plugins: [sveltekit()],

  server: {
    proxy: {
      "^/ws.*": {
        target: "ws://localhost:3000",
        ws: true,
        rewriteWsOrigin: true,
      },
      "^/api/grid.*": {
        target: "http://localhost:3000",
      },
      "^/api/subgrid.*": {
        target: "http://localhost:3000",
      },
      "^/set.*": {
        target: "http://localhost:3000",
      },
    },
  },
});
