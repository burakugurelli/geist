import { defineConfig } from "vitest/config";

function filePath(relativePath: string): string {
  const url = new URL(relativePath, import.meta.url);
  const decodedPath = decodeURIComponent(url.pathname);

  // File URLs encode spaces and represent Windows drive and UNC paths differently.
  if (url.host) {
    return `//${url.host}${decodedPath}`;
  }

  return /^\/[A-Za-z]:\//.test(decodedPath)
    ? decodedPath.slice(1)
    : decodedPath;
}

const repositoryRoot = filePath("..");
const webRoot = filePath(".");
const demoRoot = filePath("../examples/web");
const demoOutput = filePath("./demo-dist");

export default defineConfig({
  root: demoRoot,
  server: {
    fs: {
      allow: [repositoryRoot],
    },
  },
  build: {
    outDir: demoOutput,
    emptyOutDir: true,
  },
  test: {
    root: webRoot,
  },
});
