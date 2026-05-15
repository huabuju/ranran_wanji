const normalizeText = (value) => String(value ?? "").trim();

export const normalizeRomUrls = (value) => {
  if (!Array.isArray(value)) return [];
  return value.map((item) => normalizeText(item)).filter(Boolean);
};

export const getAndroidMajorVersion = (value) => {
  const matched = normalizeText(value).match(/\d+/);
  return matched ? Number(matched[0]) : 0;
};

export const normalizeAndroidLabel = (value) => {
  const normalized = normalizeText(value);
  if (!normalized) return "未知";
  if (normalized.startsWith("安卓")) return normalized.replace(/\s+/g, "");

  const matched = normalized.match(/\d+(?:\.\d+)?/);
  return matched ? `安卓${matched[0]}` : normalized;
};

const extractFileNameFromUrl = (urlValue) => {
  const primaryUrl = normalizeText(urlValue);
  if (!primaryUrl) return "";

  try {
    const url = new URL(primaryUrl);
    const parts = url.pathname.split("/");
    return decodeURIComponent(parts[parts.length - 1] || "");
  } catch {
    return "";
  }
};

const buildRomEntryKey = (entry) => ([
  normalizeText(entry.codename),
  normalizeText(entry.version),
  normalizeText(entry.flashType),
  normalizeText(entry.name),
  normalizeText(entry.date),
  normalizeText(entry.url),
].join("|"));

export const normalizeRomEntry = (entry, extra = {}) => {
  const urls = normalizeRomUrls(entry?.url);
  const primaryUrl = urls[0] || "";
  const android = normalizeText(extra.android ?? entry?.android);
  const region = normalizeText(extra.region ?? entry?.region);
  const regionLabel = normalizeText(extra.regionLabel ?? entry?.regionLabel);

  const normalizedEntry = {
    ...entry,
    ...extra,
    codename: normalizeText(extra.codename ?? entry?.codename),
    name: normalizeText(extra.name ?? entry?.name),
    brand: normalizeText(extra.brand ?? entry?.brand),
    version: normalizeText(extra.version ?? entry?.version),
    android,
    androidLabel: normalizeAndroidLabel(android),
    region,
    regionLabel,
    flashType: normalizeText(extra.flashType ?? entry?.flashType),
    date: normalizeText(extra.date ?? entry?.date),
    branch: normalizeText(extra.branch ?? entry?.branch),
    size: normalizeText(extra.size ?? entry?.size),
    filename: normalizeText(extra.filename ?? entry?.filename) || extractFileNameFromUrl(primaryUrl),
    url: primaryUrl,
    urls,
    mirrorCount: urls.length,
  };

  return {
    ...normalizedEntry,
    key: buildRomEntryKey(normalizedEntry),
  };
};
