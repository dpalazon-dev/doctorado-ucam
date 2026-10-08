import { expect, it } from "vitest";
import { createServer } from "vite";
import { localPdfResources } from "../../../../vite.config";

it("development_server_serves_only_local_pdfjs_resources", async () => {
  const server = await createServer({
    configFile: false,
    plugins: [localPdfResources()],
    server: { host: "127.0.0.1", port: 0, strictPort: false },
    logLevel: "silent",
  });
  try {
    await server.listen();
    const address = server.httpServer?.address();
    if (!address || typeof address === "string")
      throw new Error("Vite did not bind a TCP listener.");
    const response = await fetch(
      `http://127.0.0.1:${address.port}/pdfjs/standard_fonts/FoxitSerif.pfb`,
    );
    expect(response.status).toBe(200);
    expect(response.headers.get("content-type")).toContain(
      "application/octet-stream",
    );
    expect((await response.arrayBuffer()).byteLength).toBeGreaterThan(0);
    const fallback = await fetch(
      `http://127.0.0.1:${address.port}/pdfjs/wasm/openjpeg_nowasm_fallback.js`,
    );
    expect(fallback.status).toBe(200);
    expect(fallback.headers.get("content-type")).toContain("text/javascript");
    expect((await fallback.arrayBuffer()).byteLength).toBeGreaterThan(0);
    const wasm = await fetch(
      `http://127.0.0.1:${address.port}/pdfjs/wasm/openjpeg.wasm`,
    );
    if (wasm.status === 200) {
      expect(wasm.headers.get("content-type")).toContain("application/wasm");
      expect((await wasm.arrayBuffer()).byteLength).toBeGreaterThan(0);
    }
    expect(
      (
        await fetch(
          `http://127.0.0.1:${address.port}/pdfjs/%2e%2e%2Fpackage.json`,
        )
      ).status,
    ).toBe(404);
    expect(
      (await fetch(`http://127.0.0.1:${address.port}/pdfjs/quickjs/sandbox.js`))
        .status,
    ).toBe(404);
  } finally {
    await server.close();
  }
});
