import AutoImport from "unplugin-auto-import/vite";

import Components from "unplugin-vue-components/vite";
import fs from "node:fs";
import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import { fileURLToPath, URL } from "node:url";
import { ElementPlusResolver } from "unplugin-vue-components/resolvers";

const host = "127.0.0.1";
const appDateVersion = JSON.parse(fs.readFileSync(new URL("./update.json", import.meta.url), "utf-8")).dateVersion || "";

function buildInfoPlugin() {
  return {
    name: "app-build-info",
    closeBundle() {
      const distDir = fileURLToPath(new URL("./dist", import.meta.url));
      fs.mkdirSync(distDir, { recursive: true });
      fs.writeFileSync(
        new URL("./dist/build-info.json", import.meta.url),
        `${JSON.stringify({ dateVersion: appDateVersion }, null, 2)}\n`,
      );
    },
  };
}

// https://vite.dev/config/
export default defineConfig(async () => ({
  define: {
    __APP_DATE_VERSION__: JSON.stringify(String(appDateVersion)),
  },

  plugins: [
    buildInfoPlugin(),
    vue(),
    AutoImport({
      resolvers: [ElementPlusResolver()],
    }),
    Components({
      resolvers: [ElementPlusResolver()],
    }),
  ],

  resolve: {
    alias: {
      "@": fileURLToPath(new URL("./src", import.meta.url)),
    },
  },

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent Vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    host: true,
    port: 1420,
    strictPort: true,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // 3. tell Vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },
}));
