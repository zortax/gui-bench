import { defineConfig } from "vite";
import solid from "vite-plugin-solid";

// The page of the Tauri app: `npm run build` writes `dist/`, which the binary embeds.
export default defineConfig({
  plugins: [solid()],
  clearScreen: false,
  server: { port: 5173, strictPort: true },
});
