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

function getLatestVersion() {
  return new Promise((resolve, reject) => {
    https.get(
      `https://api.github.com/repos/${REPO}/releases/latest`,
      { headers: { "User-Agent": "christ-cli-npm" } },
      (res) => {
        let data = "";
        res.on("data", (chunk) => (data += chunk));
        res.on("end", () => {
          try {
            const json = JSON.parse(data);
            resolve(json.tag_name.replace("v", ""));
          } catch (e) {
            reject(e);
          }
        });
      }
    ).on("error", reject);
  });
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
        const file = fs.createWriteStream(dest);
        res.pipe(file);
        file.on("finish", () => file.close(resolve));
      }).on("error", reject);
    };
    follow(url);
  });
}

async function main() {
  const { target, isWindows } = getPlatform();
  const version = await getLatestVersion();
  const ext = isWindows ? "zip" : "tar.gz";
  const url = `https://github.com/${REPO}/releases/download/v${version}/${BINARY}-${target}.${ext}`;

  console.log(`Downloading christ-cli v${version} for ${target}...`);

  const binDir = path.join(__dirname, "bin");
  fs.mkdirSync(binDir, { recursive: true });

  const archivePath = path.join(binDir, `christ.${ext}`);
  await download(url, archivePath);

  if (isWindows) {
    execSync(`tar -xf "${archivePath}" -C "${binDir}"`, { stdio: "inherit" });
  } else {
    execSync(`tar xzf "${archivePath}" -C "${binDir}"`, { stdio: "inherit" });
    fs.chmodSync(path.join(binDir, BINARY), 0o755);
  }

  fs.unlinkSync(archivePath);
  console.log("christ-cli installed successfully!");
}

main().catch((err) => {
  console.error("Installation failed:", err.message);
  process.exit(1);
});
