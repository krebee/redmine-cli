#!/usr/bin/env node

const { existsSync } = require("node:fs");
const { dirname, join, resolve } = require("node:path");
const { spawnSync } = require("node:child_process");

const packageRoot = resolve(__dirname, "..");
const repoRoot = resolve(packageRoot, "..", "..");

function platformPackageName() {
  const platform = process.platform;
  const arch = process.arch;

  if (platform === "win32" && arch === "x64") {
    return "redmine-cli-win32-x64";
  }

  if (platform === "linux" && arch === "x64") {
    return "redmine-cli-linux-x64";
  }

  if (platform === "darwin" && arch === "arm64") {
    return "redmine-cli-darwin-arm64";
  }

  return null;
}

function binaryName() {
  return process.platform === "win32" ? "redmine-cli.exe" : "redmine-cli";
}

function candidateBinaries() {
  const envBinary = process.env.REDMINE_CLI_BIN || process.env.REDMINE_AGENT_BIN;
  const candidates = [];

  if (envBinary) {
    candidates.push(envBinary);
  }

  candidates.push(join(repoRoot, "target", "debug", binaryName()));
  candidates.push(join(repoRoot, "target", "release", binaryName()));

  const packageName = platformPackageName();
  if (packageName) {
    candidates.push(join(dirname(packageRoot), packageName, "bin", binaryName()));
  }

  return candidates;
}

const binary = candidateBinaries().find((candidate) => existsSync(candidate));

if (!binary) {
  console.error("redmine-cli binary was not found.");
  console.error("Build it with `cargo build -p redmine-cli`, or set REDMINE_CLI_BIN.");
  process.exit(1);
}

const result = spawnSync(binary, process.argv.slice(2), {
  stdio: "inherit",
});

if (result.error) {
  console.error(result.error.message);
  process.exit(1);
}

process.exit(result.status ?? 1);
