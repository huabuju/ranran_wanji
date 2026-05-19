export const PATCH_MODE_OPTIONS = [
  { label: 'Magisk', value: 'magisk' },
  { label: 'Magisk_Alpha', value: 'magisk_alpha' },
  { label: 'APatch', value: 'apatch' },
  { label: 'FolkPatch', value: 'folkpatch' },
  { label: 'KernelSU', value: 'kernelsu' },
  { label: 'KernelSU_Next', value: 'kernelsu_next' },
  { label: 'SukiSU_Ultra', value: 'sukisu_ultra' },
  { label: 'ReSukiSU', value: 'resukisu' },
];

export const APATCH_SUPER_KEY_MIN_LENGTH = 8;
export const APATCH_SUPER_KEY_MAX_LENGTH = 63;

export function normalizePatchMode(value) {
  const normalized = String(value || '').trim().toLowerCase();
  if (normalized === 'kernelsu') return 'kernelsu';
  if (normalized === 'kernelsu_next') return 'kernelsu_next';
  if (normalized === 'sukisu_ultra') return 'sukisu_ultra';
  if (normalized === 'resukisu') return 'resukisu';
  if (normalized === 'magisk_alpha') return 'magisk_alpha';
  if (normalized === 'apatch') return 'apatch';
  if (normalized === 'folkpatch') return 'folkpatch';
  return 'magisk';
}

export function isKernelSuMode(value) {
  return ['kernelsu', 'kernelsu_next', 'sukisu_ultra', 'resukisu'].includes(normalizePatchMode(value));
}

export function isApatchMode(value) {
  return ['apatch', 'folkpatch'].includes(normalizePatchMode(value));
}

export function getPatchModeLabel(mode) {
  const normalized = normalizePatchMode(mode);
  if (normalized === 'kernelsu') return 'KernelSU';
  if (normalized === 'kernelsu_next') return 'KernelSU_Next';
  if (normalized === 'sukisu_ultra') return 'SukiSU_Ultra';
  if (normalized === 'resukisu') return 'ReSukiSU';
  if (normalized === 'magisk_alpha') return 'Magisk_Alpha';
  if (normalized === 'apatch') return 'APatch';
  if (normalized === 'folkpatch') return 'FolkPatch';
  return 'Magisk';
}

export function isHttpUrl(value) {
  const normalized = String(value || '').trim().toLowerCase();
  return normalized.startsWith('http://') || normalized.startsWith('https://');
}

export function getFileTail(path) {
  const normalized = String(path || '').trim().replace(/[\\/]+$/, '');
  if (!normalized) return '';
  const parts = normalized.split(/[\\/]/).filter(Boolean);
  return parts[parts.length - 1] || normalized;
}
