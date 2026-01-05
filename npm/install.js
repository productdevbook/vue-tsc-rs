#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import os from "node:os";
import { fileURLToPath } from "node:url";
import { createWriteStream } from "node:fs";
import { pipeline } from "node:stream/promises";
import { createGunzip } from "node:zlib";
import { extract } from "tar";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const PACKAGE_NAME = "vue-tsc-rs";
const REPO = "productdevbook/vue-tsc-rs";

const PLATFORMS = {
  "darwin-arm64": { archive: "vue-tsc-rs-darwin-arm64.tar.gz", binary: "vue-tsc-rs" },
  "darwin-x64": { archive: "vue-tsc-rs-darwin-x64.tar.gz", binary: "vue-tsc-rs" },
  "linux-arm64": { archive: "vue-tsc-rs-linux-arm64.tar.gz", binary: "vue-tsc-rs" },
  "linux-x64": { archive: "vue-tsc-rs-linux-x64.tar.gz", binary: "vue-tsc-rs" },
  "win32-arm64": { archive: "vue-tsc-rs-win32-arm64.zip", binary: "vue-tsc-rs.exe" },
  "win32-x64": { archive: "vue-tsc-rs-win32-x64.zip", binary: "vue-tsc-rs.exe" },
};

function getPlatformKey() {
  const platform = os.platform();
  const arch = os.arch();

  const platformMap = { darwin: "darwin", linux: "linux", win32: "win32" };
  const archMap = { arm64: "arm64", x64: "x64", x86_64: "x64" };

  const p = platformMap[platform];
  const a = archMap[arch];

  if (!p || !a) {
    return null;
  }

  return `${p}-${a}`;
}

async function getVersion() {
  const packageJson = JSON.parse(
    fs.readFileSync(path.join(__dirname, "package.json"), "utf-8")
  );
  return packageJson.version;
}

async function downloadFile(url, dest) {
  const response = await fetch(url, { redirect: "follow" });

  if (!response.ok) {
    throw new Error(`Failed to download: ${response.status} ${response.statusText}`);
  }

  const fileStream = createWriteStream(dest);
  await pipeline(response.body, fileStream);
}

async function extractTarGz(archivePath, destDir) {
  await extract({
    file: archivePath,
    cwd: destDir,
  });
}

async function extractZip(archivePath, destDir) {
  const { execSync } = await import("node:child_process");

  if (process.platform === "win32") {
    execSync(`powershell -Command "Expand-Archive -Path '${archivePath}' -DestinationPath '${destDir}' -Force"`, {
      stdio: "pipe",
    });
  } else {
    execSync(`unzip -o "${archivePath}" -d "${destDir}"`, { stdio: "pipe" });
  }
}

async function install() {
  const platformKey = getPlatformKey();

  if (!platformKey) {
    console.error(`[${PACKAGE_NAME}] Unsupported platform: ${os.platform()}-${os.arch()}`);
    console.error("Please build from source: https://github.com/productdevbook/vue-tsc-rs");
    process.exit(1);
  }

  const platformInfo = PLATFORMS[platformKey];
  if (!platformInfo) {
    console.error(`[${PACKAGE_NAME}] No binary available for ${platformKey}`);
    process.exit(1);
  }

  const version = await getVersion();
  const binDir = path.join(__dirname, "bin");
  const binaryPath = path.join(binDir, platformInfo.binary);

  // Check if already installed
  if (fs.existsSync(binaryPath)) {
    console.log(`[${PACKAGE_NAME}] Binary already exists`);
    return;
  }

  console.log(`[${PACKAGE_NAME}] Downloading binary for ${platformKey}...`);

  const downloadUrl = `https://github.com/${REPO}/releases/download/v${version}/${platformInfo.archive}`;

  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "vue-tsc-rs-"));
  const archivePath = path.join(tempDir, platformInfo.archive);

  try {
    await downloadFile(downloadUrl, archivePath);

    fs.mkdirSync(binDir, { recursive: true });

    if (platformInfo.archive.endsWith(".tar.gz")) {
      await extractTarGz(archivePath, binDir);
    } else {
      await extractZip(archivePath, binDir);
    }

    // Make executable on Unix
    if (process.platform !== "win32") {
      fs.chmodSync(binaryPath, 0o755);
    }

    console.log(`[${PACKAGE_NAME}] Binary installed successfully`);
  } catch (error) {
    console.error(`[${PACKAGE_NAME}] Failed to download binary: ${error.message}`);
    console.error("");
    console.error("You can build from source:");
    console.error("  git clone https://github.com/productdevbook/vue-tsc-rs");
    console.error("  cd vue-tsc-rs && cargo install --path crates/vue-tsc-rs");
    process.exit(1);
  } finally {
    // Cleanup temp directory
    fs.rmSync(tempDir, { recursive: true, force: true });
  }
}

install();
