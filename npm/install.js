#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import os from "node:os";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const PACKAGE_NAME = "vue-tsc-rs";
const BINARY_NAME = process.platform === "win32" ? "vue-tsc-rs.exe" : "vue-tsc-rs";

/**
 * Get the platform-specific package name
 */
function getPlatformPackage() {
  const platform = os.platform();
  const arch = os.arch();

  const platformMap = {
    darwin: "darwin",
    linux: "linux",
    win32: "win32",
  };

  const archMap = {
    arm64: "arm64",
    x64: "x64",
    x86_64: "x64",
  };

  const platformName = platformMap[platform];
  const archName = archMap[arch];

  if (!platformName || !archName) {
    console.error(`Unsupported platform: ${platform}-${arch}`);
    console.error("Please build from source: https://github.com/productdevbook/vue-tsc-rs");
    process.exit(1);
  }

  return `@vue-tsc-rs/${platformName}-${archName}`;
}

/**
 * Find the binary in node_modules
 */
function findBinary(packageName) {
  const possiblePaths = [
    path.join(__dirname, "node_modules", packageName, BINARY_NAME),
    path.join(__dirname, "..", packageName, BINARY_NAME),
    path.join(__dirname, "..", "..", packageName, BINARY_NAME),
  ];

  for (const binaryPath of possiblePaths) {
    if (fs.existsSync(binaryPath)) {
      return binaryPath;
    }
  }

  return null;
}

/**
 * Create the bin directory and wrapper script
 */
function createBinWrapper(binaryPath) {
  const binDir = path.join(__dirname, "bin");

  if (!fs.existsSync(binDir)) {
    fs.mkdirSync(binDir, { recursive: true });
  }

  const wrapperPath = path.join(binDir, BINARY_NAME);

  if (process.platform === "win32") {
    const cmdWrapper = `@echo off\n"${binaryPath}" %*`;
    fs.writeFileSync(wrapperPath + ".cmd", cmdWrapper);

    const shWrapper = `#!/bin/sh\nexec "${binaryPath.replace(/\\/g, "/")}" "$@"`;
    fs.writeFileSync(wrapperPath, shWrapper);
  } else {
    const shWrapper = `#!/bin/sh\nexec "${binaryPath}" "$@"`;
    fs.writeFileSync(wrapperPath, shWrapper);
    fs.chmodSync(wrapperPath, 0o755);
  }

  console.log(`[${PACKAGE_NAME}] Binary installed successfully`);
}

/**
 * Main installation function
 */
function install() {
  const packageName = getPlatformPackage();

  console.log(`[${PACKAGE_NAME}] Looking for ${packageName}...`);

  const binaryPath = findBinary(packageName);

  if (!binaryPath) {
    console.error(`[${PACKAGE_NAME}] Could not find binary for ${packageName}`);
    console.error("");
    console.error("This might happen if:");
    console.error("  1. Your platform is not supported");
    console.error("  2. The optional dependency was not installed");
    console.error("");
    console.error("To build from source:");
    console.error("  git clone https://github.com/productdevbook/vue-tsc-rs");
    console.error("  cd vue-tsc-rs && cargo install --path crates/vue-tsc-rs");
    process.exit(1);
  }

  console.log(`[${PACKAGE_NAME}] Found binary at ${binaryPath}`);
  createBinWrapper(binaryPath);
}

install();
