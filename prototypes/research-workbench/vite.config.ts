import { defineConfig } from "vite";
import type { Plugin } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
// @ts-expect-error Vite loads this configuration in Node; Node types are not a product dependency.
import { readdirSync, readFileSync } from "node:fs";
// @ts-expect-error Vite loads this configuration in Node; Node types are not a product dependency.
import { resolve, relative, sep } from "node:path";

const PDF_RESOURCE_DIRECTORIES = ["cmaps", "standard_fonts", "iccs", "wasm"];

function localFiles(directory: string): string[] {
  const entries = readdirSync(directory, { withFileTypes: true }) as Array<{
    name: string;
    isDirectory(): boolean;
  }>;
  return entries
    .sort((left, right) => left.name.localeCompare(right.name))
    .flatMap((entry) => {
      const path = resolve(directory, entry.name);
      return entry.isDirectory() ? localFiles(path) : [path];
    });
}

function localPdfResources(): Plugin {
  const sourceRoot = resolve("node_modules/pdfjs-dist");
  const resourceFiles = new Map<string, string>();
  for (const directory of PDF_RESOURCE_DIRECTORIES) {
    for (const path of localFiles(resolve(sourceRoot, directory))) {
      const name = relative(sourceRoot, path).split(sep).join("/");
      if (!name.toLowerCase().includes("quickjs"))
        resourceFiles.set(name, path);
    }
  }
  resourceFiles.set("LICENSE", resolve(sourceRoot, "LICENSE"));

  return {
    name: "local-pdfjs-resources",
    configureServer(server) {
      server.middlewares.use((request, response, next) => {
        const incoming = request as typeof request & {
          method?: string;
          url?: string;
        };
        if (incoming.method !== "GET" || !incoming.url?.startsWith("/pdfjs/")) {
          next();
          return;
        }
        let name: string;
        try {
          name = decodeURIComponent(
            incoming.url.split("?", 1)[0].slice("/pdfjs/".length),
          );
        } catch {
          response.statusCode = 400;
          response.end();
          return;
        }
        const path = resourceFiles.get(name);
        if (!path) {
          response.statusCode = 404;
          response.end();
          return;
        }
        response.statusCode = 200;
        response.setHeader(
          "Content-Type",
          name === "LICENSE"
            ? "text/plain; charset=utf-8"
            : name.endsWith(".js")
              ? "text/javascript; charset=utf-8"
              : name.endsWith(".wasm")
                ? "application/wasm"
                : "application/octet-stream",
        );
        response.setHeader("Cache-Control", "no-store");
        response.end(readFileSync(path));
      });
    },
    generateBundle() {
      for (const [name, path] of resourceFiles) {
        this.emitFile({
          type: "asset",
          fileName: `pdfjs/${name}`,
          source: readFileSync(path),
        });
      }
    },
  };
}

export { localPdfResources };
export default defineConfig({
  plugins: [react(), tailwindcss(), localPdfResources()],
  clearScreen: false,
  server: { port: 1420, strictPort: true },
  envPrefix: ["VITE_", "TAURI_ENV_*"],
  build: { target: "chrome120" },
});
