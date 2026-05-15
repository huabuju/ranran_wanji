const fs = require('fs');
const path = require('path');
const archiver = require('archiver');

const ROOT_DIR = path.resolve(__dirname, '..');
const pkg = JSON.parse(fs.readFileSync(path.join(ROOT_DIR, 'package.json'), 'utf-8'));
const VERSION = pkg.version;
const APP_NAME = 'RanranToolkit';
const EXE_SRC = path.join(ROOT_DIR, 'src-tauri', 'target', 'release', `${APP_NAME}.exe`);
const OUTPUT_DIR = path.join(ROOT_DIR, 'src-tauri', 'target', 'release', 'bundle', 'portable');
const ZIP_NAME = `${APP_NAME}-v${VERSION}-bootstrap-windows-x86_64.zip`;
const ZIP_PATH = path.join(OUTPUT_DIR, ZIP_NAME);

async function buildZip() {
  if (!fs.existsSync(EXE_SRC)) {
    console.error(`[error] Missing release executable: ${EXE_SRC}`);
    console.error('Run "yarn tauri build" before creating the portable package.');
    process.exit(1);
  }

  fs.mkdirSync(OUTPUT_DIR, { recursive: true });
  if (fs.existsSync(ZIP_PATH)) {
    fs.unlinkSync(ZIP_PATH);
  }

  return new Promise((resolve, reject) => {
    const output = fs.createWriteStream(ZIP_PATH);
    const archive = archiver('zip', { zlib: { level: 9 } });

    output.on('close', resolve);
    archive.on('warning', (error) => {
      if (error.code === 'ENOENT') {
        console.warn(`[warn] ${error.message}`);
      } else {
        reject(error);
      }
    });
    archive.on('error', reject);

    archive.pipe(output);
    archive.file(EXE_SRC, { name: `${APP_NAME}.exe` });
    archive.finalize();
  });
}

buildZip()
  .then(() => {
    const sizeKB = (fs.statSync(ZIP_PATH).size / 1024).toFixed(1);
    console.log(`[done] Portable bootstrap package created: ${ZIP_PATH}`);
    console.log(`[size] ${sizeKB} KB`);
  })
  .catch((error) => {
    console.error('[error] Failed to build portable package:', error);
    process.exit(1);
  });
