const fs = require("fs");
const path = require("path");
const crypto = require("crypto");
const archiver = require("archiver");

const ROOT_DIR = path.resolve(__dirname, "..");
const pkg = JSON.parse(
  fs.readFileSync(path.join(ROOT_DIR, "package.json"), "utf-8"),
);
const VERSION = pkg.version;
const BIN_DIR = path.join(ROOT_DIR, "bin");
const OUTPUT_DIR = path.join(BIN_DIR, "cloud-parts");
const TEMP_DIR = path.join(ROOT_DIR, ".runtime-build");
const ARCHIVE_NAME = "bin-runtime.zip";
const ARCHIVE_PATH = path.join(TEMP_DIR, ARCHIVE_NAME);
const PART_SIZE = 8 * 1024 * 1024;
const DEFAULT_BASE_URL =
  "https://gitee.com/xiaowan12/ranran-toolkit-bin/raw/master/cloud-parts";
const BASE_URL = (process.env.RUNTIME_BASE_URL || DEFAULT_BASE_URL).replace(
  /\/$/,
  "",
);
const REQUIRED_FILES = [
  "platform-tools/adb.exe",
  "platform-tools/fastboot.exe",
  "aria2-core/aria2c.exe",
  "scrcpy-core/scrcpy.exe",
  "link-dumper/link-dumper.exe",
];

function ensureDir(dirPath) {
  fs.mkdirSync(dirPath, { recursive: true });
}

function resetDir(dirPath) {
  fs.rmSync(dirPath, { recursive: true, force: true });
  fs.mkdirSync(dirPath, { recursive: true });
}

function sha256File(filePath) {
  const hash = crypto.createHash("sha256");
  const fileBuffer = fs.readFileSync(filePath);
  hash.update(fileBuffer);
  return hash.digest("hex");
}

function walkDirectory(sourceDir, visit, base = sourceDir) {
  for (const entry of fs.readdirSync(sourceDir, { withFileTypes: true })) {
    const fullPath = path.join(sourceDir, entry.name);
    const relativePath = path.relative(base, fullPath);
    visit(fullPath, relativePath, entry);
    if (entry.isDirectory()) {
      walkDirectory(fullPath, visit, base);
    }
  }
}

function addBinToArchive(archive) {
  walkDirectory(BIN_DIR, (fullPath, relativePath, entry) => {
    const normalizedRelative = relativePath.split(path.sep).join("/");
    if (!normalizedRelative) return;
    if (
      normalizedRelative === "cloud-parts" ||
      normalizedRelative.startsWith("cloud-parts/")
    ) {
      return;
    }

    const archivePath = `bin/${normalizedRelative}`;
    if (entry.isDirectory()) {
      archive.append("", { name: `${archivePath}/` });
    } else {
      archive.file(fullPath, { name: archivePath });
    }
  });
}

async function createArchive() {
  resetDir(TEMP_DIR);
  return new Promise((resolve, reject) => {
    const output = fs.createWriteStream(ARCHIVE_PATH);
    const archive = archiver("zip", { zlib: { level: 9 } });

    output.on("close", resolve);
    archive.on("warning", (error) => {
      if (error.code === "ENOENT") {
        console.warn(`[warn] ${error.message}`);
      } else {
        reject(error);
      }
    });
    archive.on("error", reject);

    archive.pipe(output);
    addBinToArchive(archive);
    archive.finalize();
  });
}

function splitArchive() {
  resetDir(OUTPUT_DIR);
  const archiveBuffer = fs.readFileSync(ARCHIVE_PATH);
  const partFiles = [];

  for (
    let offset = 0, index = 1;
    offset < archiveBuffer.length;
    offset += PART_SIZE, index += 1
  ) {
    const chunk = archiveBuffer.subarray(offset, offset + PART_SIZE);
    const partName = `${ARCHIVE_NAME}.${String(index).padStart(3, "0")}`;
    const partPath = path.join(OUTPUT_DIR, partName);
    fs.writeFileSync(partPath, chunk);
    partFiles.push({
      name: partName,
      size: chunk.length,
      sha256: sha256File(partPath),
    });
  }

  return partFiles;
}

function writeManifest(parts) {
  const archiveSize = fs.statSync(ARCHIVE_PATH).size;
  const manifest = {
    version: VERSION,
    archiveName: ARCHIVE_NAME,
    archiveSize,
    archiveSha256: sha256File(ARCHIVE_PATH),
    baseUrl: BASE_URL,
    requiredFiles: REQUIRED_FILES,
    parts,
  };

  fs.writeFileSync(
    path.join(OUTPUT_DIR, "runtime-manifest.json"),
    `${JSON.stringify(manifest, null, 2)}\n`,
    "utf-8",
  );
}

async function main() {
  if (!fs.existsSync(BIN_DIR)) {
    console.error(`[error] Missing bin directory: ${BIN_DIR}`);
    process.exit(1);
  }

  ensureDir(OUTPUT_DIR);
  await createArchive();
  const parts = splitArchive();
  writeManifest(parts);

  console.log(`[done] Runtime archive created: ${ARCHIVE_PATH}`);
  console.log(
    `[done] Runtime manifest created: ${path.join(OUTPUT_DIR, "runtime-manifest.json")}`,
  );
  console.log(`[parts] ${parts.length} file(s) written to ${OUTPUT_DIR}`);
}

main().catch((error) => {
  console.error("[error] Failed to build runtime assets:", error);
  process.exit(1);
});
