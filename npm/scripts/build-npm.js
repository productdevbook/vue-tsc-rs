#!/usr/bin/env node

/**
 * Build script for creating npm packages from Rust binaries
 *
 * Usage:
 *   node scripts/build-npm.js --target darwin-arm64 --binary ./path/to/vue-tsc-rs
 *   node scripts/build-npm.js --all --artifacts ./artifacts
 */

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const packageJson = JSON.parse(
  fs.readFileSync(path.join(__dirname, "..", "package.json"), "utf-8")
);

const PACKAGE_VERSION = packageJson.version;
const PACKAGE_NAME = "vue-tsc-rs";

const PLATFORMS = [
  { name: "darwin-arm64", os: "darwin", cpu: "arm64", ext: "" },
  { name: "darwin-x64", os: "darwin", cpu: "x64", ext: "" },
  { name: "linux-arm64", os: "linux", cpu: "arm64", ext: "" },
  { name: "linux-x64", os: "linux", cpu: "x64", ext: "" },
  { name: "win32-arm64", os: "win32", cpu: "arm64", ext: ".exe" },
  { name: "win32-x64", os: "win32", cpu: "x64", ext: ".exe" },
];

/**
 * Create a platform-specific package
 */
function createPlatformPackage(platform, binaryPath, outDir) {
  const packageName = `@vue-tsc-rs/${platform.name}`;
  const packageDir = path.join(outDir, platform.name);

  fs.mkdirSync(packageDir, { recursive: true });

  const platformPackageJson = {
    name: packageName,
    version: PACKAGE_VERSION,
    description: `${PACKAGE_NAME} binary for ${platform.name}`,
    license: "MIT",
    repository: {
      type: "git",
      url: "git+https://github.com/productdevbook/vue-tsc-rs.git",
    },
    os: [platform.os],
    cpu: [platform.cpu],
    files: [`vue-tsc-rs${platform.ext}`],
  };

  fs.writeFileSync(
    path.join(packageDir, "package.json"),
    JSON.stringify(platformPackageJson, null, 2)
  );

  const binaryName = `vue-tsc-rs${platform.ext}`;
  const destPath = path.join(packageDir, binaryName);

  if (binaryPath && fs.existsSync(binaryPath)) {
    fs.copyFileSync(binaryPath, destPath);
    if (platform.ext === "") {
      fs.chmodSync(destPath, 0o755);
    }
    console.log(`Created ${packageName} with binary from ${binaryPath}`);
  } else {
    console.log(`Created ${packageName} (no binary yet)`);
  }

  return packageDir;
}

/**
 * Parse command line arguments
 */
function parseArgs() {
  const args = process.argv.slice(2);
  const options = {
    target: null,
    binary: null,
    all: false,
    artifacts: null,
    outDir: path.join(__dirname, "..", "dist"),
  };

  for (let i = 0; i < args.length; i++) {
    switch (args[i]) {
      case "--target":
        options.target = args[++i];
        break;
      case "--binary":
        options.binary = args[++i];
        break;
      case "--all":
        options.all = true;
        break;
      case "--artifacts":
        options.artifacts = args[++i];
        break;
      case "--out":
        options.outDir = args[++i];
        break;
      case "--help":
        console.log(`
Usage: build-npm.js [options]

Options:
  --target <platform>   Build for specific platform (e.g., darwin-arm64)
  --binary <path>       Path to the compiled binary
  --all                 Build all platform packages
  --artifacts <dir>     Directory containing all platform binaries
  --out <dir>           Output directory (default: dist)
  --help                Show this help
        `);
        process.exit(0);
    }
  }

  return options;
}

/**
 * Main entry point
 */
function main() {
  const options = parseArgs();

  fs.mkdirSync(options.outDir, { recursive: true });

  if (options.all) {
    for (const platform of PLATFORMS) {
      let binaryPath = null;

      if (options.artifacts) {
        const possiblePaths = [
          path.join(options.artifacts, platform.name, `vue-tsc-rs${platform.ext}`),
          path.join(options.artifacts, `vue-tsc-rs-${platform.name}${platform.ext}`),
        ];

        for (const p of possiblePaths) {
          if (fs.existsSync(p)) {
            binaryPath = p;
            break;
          }
        }
      }

      createPlatformPackage(platform, binaryPath, options.outDir);
    }

    const mainPackageDir = path.join(options.outDir, PACKAGE_NAME);
    fs.mkdirSync(mainPackageDir, { recursive: true });

    const filesToCopy = ["package.json", "install.js", "index.js", "index.d.ts", "README.md"];
    for (const file of filesToCopy) {
      const src = path.join(__dirname, "..", file);
      const dest = path.join(mainPackageDir, file);
      if (fs.existsSync(src)) {
        fs.copyFileSync(src, dest);
      }
    }

    fs.mkdirSync(path.join(mainPackageDir, "bin"), { recursive: true });
    fs.writeFileSync(path.join(mainPackageDir, "bin", ".gitkeep"), "");

    console.log(`\nAll packages created in ${options.outDir}`);
  } else if (options.target) {
    const platform = PLATFORMS.find((p) => p.name === options.target);
    if (!platform) {
      console.error(`Unknown platform: ${options.target}`);
      console.error(`Valid platforms: ${PLATFORMS.map((p) => p.name).join(", ")}`);
      process.exit(1);
    }

    createPlatformPackage(platform, options.binary, options.outDir);
  } else {
    console.error("Please specify --target <platform> or --all");
    process.exit(1);
  }
}

main();
