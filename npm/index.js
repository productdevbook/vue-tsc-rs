#!/usr/bin/env node

import { spawn } from "node:child_process";
import path from "node:path";
import os from "node:os";
import fs from "node:fs";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

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

  return `@vue-tsc-rs/${platformMap[platform]}-${archMap[arch]}`;
}

/**
 * Find the binary path
 */
export function getBinaryPath() {
  const packageName = getPlatformPackage();

  const wrapperPath = path.join(__dirname, "bin", BINARY_NAME);
  if (fs.existsSync(wrapperPath)) {
    return wrapperPath;
  }

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

  throw new Error(
    `Could not find vue-tsc-rs binary. Make sure the package is installed correctly.`
  );
}

/**
 * Run vue-tsc-rs with the given arguments
 * @param {string[]} args - Command line arguments
 * @param {object} options - Spawn options
 * @returns {Promise<{code: number, stdout: string, stderr: string}>}
 */
export function run(args = [], options = {}) {
  return new Promise((resolve, reject) => {
    const binaryPath = getBinaryPath();

    const child = spawn(binaryPath, args, {
      stdio: options.stdio || "inherit",
      cwd: options.cwd || process.cwd(),
      env: { ...process.env, ...options.env },
    });

    let stdout = "";
    let stderr = "";

    if (options.stdio === "pipe") {
      child.stdout?.on("data", (data) => {
        stdout += data.toString();
      });
      child.stderr?.on("data", (data) => {
        stderr += data.toString();
      });
    }

    child.on("error", reject);
    child.on("close", (code) => {
      resolve({ code: code || 0, stdout, stderr });
    });
  });
}

/**
 * Check types in a Vue project
 * @param {string} workspace - Path to the workspace
 * @param {object} options - Check options
 * @returns {Promise<{code: number, stdout: string, stderr: string}>}
 */
export async function check(workspace, options = {}) {
  const args = ["--workspace", workspace];

  if (options.project) {
    args.push("--project", options.project);
  }

  if (options.output) {
    args.push("--output", options.output);
  }

  if (options.failOnWarning) {
    args.push("--fail-on-warning");
  }

  if (options.skipTypecheck) {
    args.push("--skip-typecheck");
  }

  if (options.verbose) {
    args.push("--verbose");
  }

  return run(args, { stdio: options.stdio || "pipe", cwd: workspace });
}

// CLI entry point
const isMain = process.argv[1] === __filename ||
               process.argv[1] === fileURLToPath(import.meta.url);

if (isMain) {
  run(process.argv.slice(2)).then(({ code }) => {
    process.exit(code);
  }).catch((error) => {
    console.error(error.message);
    process.exit(1);
  });
}
