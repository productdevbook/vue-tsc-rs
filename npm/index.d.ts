export interface RunOptions {
  stdio?: "inherit" | "pipe" | "ignore";
  cwd?: string;
  env?: Record<string, string>;
}

export interface RunResult {
  code: number;
  stdout: string;
  stderr: string;
}

export interface CheckOptions {
  project?: string;
  output?: "human" | "human-verbose" | "json" | "machine";
  failOnWarning?: boolean;
  skipTypecheck?: boolean;
  verbose?: boolean;
  stdio?: "inherit" | "pipe" | "ignore";
}

/**
 * Get the path to the vue-tsc-rs binary
 */
export function getBinaryPath(): string;

/**
 * Run vue-tsc-rs with custom arguments
 */
export function run(args?: string[], options?: RunOptions): Promise<RunResult>;

/**
 * Check types in a Vue project
 */
export function check(workspace: string, options?: CheckOptions): Promise<RunResult>;
