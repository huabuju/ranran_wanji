const fs = require('fs');
const path = require('path');

const ROOT_DIR = path.resolve(__dirname, '..');
const UPDATE_JSON_PATH = path.join(ROOT_DIR, 'update.json');

function pad(value) {
  return String(value).padStart(2, '0');
}

function formatDateVersion(date) {
  return [
    date.getFullYear(),
    pad(date.getMonth() + 1),
    pad(date.getDate()),
    pad(date.getHours()),
    pad(date.getMinutes()),
    pad(date.getSeconds()),
  ].join('');
}

const updateInfo = JSON.parse(fs.readFileSync(UPDATE_JSON_PATH, 'utf-8'));
updateInfo.dateVersion = formatDateVersion(new Date());

fs.writeFileSync(UPDATE_JSON_PATH, `${JSON.stringify(updateInfo, null, 2)}\n`);
console.log(`[done] update.json dateVersion updated: ${updateInfo.dateVersion}`);
