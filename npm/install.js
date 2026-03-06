const { execSync } = require("child_process");
const fs = require("fs");
const path = require("path");
const https = require("https");
const os = require("os");

const REPO = "whoisyurii/christ-cli";
const BINARY = "christ";

function getPlatform() {
  const platform = os.platform();
  const arch = os.arch();

  const targets = {
    "darwin-x64": "x86_64-apple-darwin",
    "darwin-arm64": "aarch64-apple-darwin",
    "linux-x64": "x86_64-unknown-linux-gnu",
    "linux-arm64": "aarch64-unknown-linux-gnu",
    "win32-x64": "x86_64-pc-windows-msvc",
  };

  const key = `${platform}-${arch}`;
  const target = targets[key];

  if (!target) {
    console.error(`Unsupported platform: ${key}`);
    process.exit(1);
  }

  return { target, isWindows: platform === "win32" };
}

function getVersion() {
  const pkg = require(path.join(__dirname, "package.json"));
  return pkg.version;
}

function download(url, dest) {
  return new Promise((resolve, reject) => {
    const follow = (url) => {
      https.get(url, { headers: { "User-Agent": "christ-cli-npm" } }, (res) => {
        if (res.statusCode >= 300 && res.statusCode < 400 && res.headers.location) {
          follow(res.headers.location);
          return;
        }
        if (res.statusCode !== 200) {
          reject(new Error(`Download failed: HTTP ${res.statusCode}`));
          return;
        }

        const total = parseInt(res.headers["content-length"], 10) || 0;
        let downloaded = 0;
        const file = fs.createWriteStream(dest);

        res.on("data", (chunk) => {
          downloaded += chunk.length;
          file.write(chunk);
          if (total > 0) {
            const pct = Math.round((downloaded / total) * 100);
            const mb = (downloaded / 1024 / 1024).toFixed(1);
            const totalMb = (total / 1024 / 1024).toFixed(1);
            const barLen = 24;
            const filled = Math.round((downloaded / total) * barLen);
            const bar = "\u2588".repeat(filled) + "\u2591".repeat(barLen - filled);
            process.stderr.write(`\r  [${bar}] ${mb}MB / ${totalMb}MB  ${pct}%`);
          }
        });

        res.on("end", () => {
          file.end();
          if (total > 0) process.stderr.write("\n");
          resolve();
        });
      }).on("error", reject);
    };
    follow(url);
  });
}

async function main() {
  const { target, isWindows } = getPlatform();
  const version = getVersion();
  const ext = isWindows ? "zip" : "tar.gz";
  const url = `https://github.com/${REPO}/releases/download/v${version}/${BINARY}-${target}.${ext}`;

  console.log(`\n  Installing christ-cli v${version} for ${target}...\n`);

  const binDir = path.join(__dirname, "bin");
  fs.mkdirSync(binDir, { recursive: true });

  const archivePath = path.join(binDir, `christ.${ext}`);
  await download(url, archivePath);

  console.log("  Extracting...");

  if (isWindows) {
    execSync(`tar -xf "${archivePath}" -C "${binDir}"`, { stdio: "pipe" });
  } else {
    execSync(`tar xzf "${archivePath}" -C "${binDir}"`, { stdio: "pipe" });
    fs.chmodSync(path.join(binDir, BINARY), 0o755);
  }

  fs.unlinkSync(archivePath);
  console.log("  christ-cli installed successfully! Run: christ\n");
}

main().catch((err) => {
  console.error("\n  Installation failed:", err.message);
  process.exit(1);
});
