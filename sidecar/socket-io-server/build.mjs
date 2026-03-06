import { build } from "esbuild";
import { execSync } from "child_process";
import { copyFileSync, mkdirSync, existsSync, writeFileSync, readFileSync } from "fs";
import { join, resolve } from "path";
import { platform, arch } from "os";

const projectRoot = resolve(import.meta.dirname);
const tauriBindaries = resolve(projectRoot, "../../src-tauri/binaries");

// Step 1: Bundle TypeScript into a single CJS file
console.log("Bundling with esbuild...");
await build({
  entryPoints: [join(projectRoot, "src/index.ts")],
  bundle: true,
  platform: "node",
  target: "node20",
  format: "cjs",
  outfile: join(projectRoot, "dist/index.cjs"),
  minify: true,
  sourcemap: false,
});
console.log("Bundle created: dist/index.cjs");

// Step 2: Determine target triple for Tauri sidecar naming
function getTargetTriple() {
  const p = platform();
  const a = arch();
  if (p === "win32") return "x86_64-pc-windows-msvc";
  if (p === "darwin" && a === "arm64") return "aarch64-apple-darwin";
  if (p === "darwin") return "x86_64-apple-darwin";
  return "x86_64-unknown-linux-gnu";
}

const triple = getTargetTriple();
const ext = platform() === "win32" ? ".exe" : "";
const binaryName = `socket-io-server-${triple}${ext}`;

// Step 3: Build Node.js SEA (Single Executable Application)
console.log("Building Node.js Single Executable Application...");

const seaConfig = {
  main: join(projectRoot, "dist/index.cjs"),
  output: join(projectRoot, "dist/sea-prep.blob"),
  disableExperimentalSEAWarning: true,
};

writeFileSync(join(projectRoot, "dist/sea-config.json"), JSON.stringify(seaConfig));

try {
  // Generate the SEA blob
  execSync(`node --experimental-sea-config dist/sea-config.json`, {
    cwd: projectRoot,
    stdio: "inherit",
  });

  // Copy the node executable
  const nodeExe = process.execPath;
  const outputPath = join(tauriBindaries, binaryName);

  if (!existsSync(tauriBindaries)) {
    mkdirSync(tauriBindaries, { recursive: true });
  }

  copyFileSync(nodeExe, outputPath);

  // Inject the SEA blob
  if (platform() === "win32") {
    // On Windows, use postject via npx
    execSync(
      `npx --yes postject "${outputPath}" NODE_SEA_BLOB dist/sea-prep.blob --sentinel-fuse NODE_SEA_FUSE_fce680ab2cc467b6e072b8b5df1996b2 --overwrite`,
      { cwd: projectRoot, stdio: "inherit" }
    );
  } else {
    execSync(
      `npx --yes postject "${outputPath}" NODE_SEA_BLOB dist/sea-prep.blob --sentinel-fuse NODE_SEA_FUSE_fce680ab2cc467b6e072b8b5df1996b2 --overwrite`,
      { cwd: projectRoot, stdio: "inherit" }
    );
    // Make executable on Unix
    execSync(`chmod +x "${outputPath}"`);
  }

  console.log(`SEA binary created: ${outputPath}`);
} catch (e) {
  console.error("SEA build failed. Falling back to bundled JS only.");
  console.error("For development, use: node dist/index.cjs --port 4849");
  console.error(e.message);
  process.exit(1);
}
