#!/usr/bin/env node
const { execFileSync } = require("child_process");
const path = require("path");
const os = require("os");

const ext = os.platform() === "win32" ? ".exe" : "";
const binary = path.join(__dirname, `christ${ext}`);

try {
  execFileSync(binary, process.argv.slice(2), { stdio: "inherit" });
} catch (err) {
  if (err.status !== null) {
    process.exit(err.status);
  }
  throw err;
}
