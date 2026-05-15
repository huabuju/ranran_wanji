/**
 * ROM 在线数据索引
 * 统一适配多数据来源，输出页面所需的品牌 / 机型 / ROM 结构
 */

import { fetchHyperOsFansCatalog, fetchHyperOsFansModelRoms } from '@/api/hyperosFans';
import { fetchMiuierCatalog, fetchMiuierModelRoms } from '@/api/miuier';
import { fetchXfuCatalog, fetchXfuModelRoms } from '@/api/xfu';
import { fetchXiaomiRomCatalog, fetchXiaomiRomModelRoms, resolveXiaomiRomDownloadUrls } from '@/api/xiaomirom';
import { normalizeRomEntry } from '@/utils/romData.js';

const BRAND_META = {
  xiaomi: { name: "小米 (Xiaomi)", color: "#ff6900", order: 0 },
  redmi: { name: "红米 (Redmi)", color: "#e63946", order: 1 },
  poco: { name: "POCO", color: "#facc15", order: 2 },
  vivo: { name: "vivo", color: "#4f46e5", order: 3 },
  oppo: { name: "OPPO", color: "#10b981", order: 4 },
  oneplus: { name: "一加 (OnePlus)", color: "#10b981", order: 5 },
  huawei: { name: "华为 (Huawei)", color: "#10b981", order: 6 },
  other: { name: "其他机型", color: "#94a3b8", order: 99 },
};

export const DEFAULT_ROM_DATA_SOURCE = 'xiaomirom';

const ROM_DATA_SOURCE_META = {
  xiaomirom: {
    key: 'xiaomirom',
    label: 'XiaomiROM',
    description: '通过 xiaomirom.com 页面抓取品牌目录、机型 ROM 列表，并在下载前继续解析真实直链。',
    supportsCatalogRomCount: true,
  },
  hyperos_fans: {
    key: 'hyperos_fans',
    label: 'HyperOS.fans',
    description: '直接读取 HyperOS.fans 公开 JSON 数据，机型 ROM 总数会在进入机型后按需加载。',
    supportsCatalogRomCount: false,
  },
  miuier: {
    key: 'miuier',
    label: 'MIUIER',
    description: '读取 roms.miuier.com 的 Nuxt payload 数据，目录和 ROM 列表都来自结构化源，下载链接可直接使用。',
    supportsCatalogRomCount: false,
  },
  xfu: {
    key: 'xfu',
    label: 'XMFirmware',
    description: '读取时或许需要科学上网，目录来自设备索引。',
    supportsCatalogRomCount: false,
  },
};

const ROM_DATA_SOURCE_HANDLERS = {
  xiaomirom: {
    loadCatalog: fetchXiaomiRomCatalog,
    loadModelRoms: (model) => fetchXiaomiRomModelRoms(String(model?.sourceRef || model?.seriesUrl || '').trim()),
    resolveRomUrls: (rom) => resolveXiaomiRomDownloadUrls(String(rom?.sourceUrl || rom?.pageUrl || rom?.url || '').trim()),
  },
  hyperos_fans: {
    loadCatalog: fetchHyperOsFansCatalog,
    loadModelRoms: (model) => fetchHyperOsFansModelRoms(String(model?.codename || '').trim()),
    resolveRomUrls: async (rom) => dedupeUrls(rom?.urls || [rom?.url]),
  },
  miuier: {
    loadCatalog: fetchMiuierCatalog,
    loadModelRoms: (model) => fetchMiuierModelRoms(String(model?.codename || '').trim()),
    resolveRomUrls: async (rom) => dedupeUrls(rom?.urls || [rom?.url]),
  },
  xfu: {
    loadCatalog: fetchXfuCatalog,
    loadModelRoms: (model) => fetchXfuModelRoms(String(model?.codename || '').trim()),
    resolveRomUrls: async (rom) => dedupeUrls(rom?.urls || [rom?.url]),
  },
};

const createSourceState = () => ({
  brandsPromise: null,
  modelRomsPromiseMap: new Map(),
  modelRomsCache: new Map(),
  romUrlPromiseMap: new Map(),
  romUrlCache: new Map(),
});

const sourceStateMap = new Map();

const normalizeSourceKey = (sourceKey) => (
  ROM_DATA_SOURCE_HANDLERS[sourceKey] ? sourceKey : DEFAULT_ROM_DATA_SOURCE
);

const getSourceState = (sourceKey) => {
  const normalizedSourceKey = normalizeSourceKey(sourceKey);
  if (!sourceStateMap.has(normalizedSourceKey)) {
    sourceStateMap.set(normalizedSourceKey, createSourceState());
  }
  return sourceStateMap.get(normalizedSourceKey);
};

const getSourceHandler = (sourceKey) => ROM_DATA_SOURCE_HANDLERS[normalizeSourceKey(sourceKey)];

export const getRomDataSourceMeta = (sourceKey) => (
  ROM_DATA_SOURCE_META[normalizeSourceKey(sourceKey)] || ROM_DATA_SOURCE_META[DEFAULT_ROM_DATA_SOURCE]
);

export const getRomDataSourceOptions = () => Object.values(ROM_DATA_SOURCE_META);

const normalizeRomList = (items, extra = {}) =>
  (Array.isArray(items) ? items : [])
    .map((item) => normalizeRomEntry(item, extra))
    .filter((item) => item.url || item.sourceUrl);

const inferBrandKey = (name, explicitBrand = "") => {
  const normalizedBrand = String(explicitBrand || "").trim().toLowerCase();
  if (normalizedBrand) return normalizedBrand;

  const lowerName = String(name || "").trim().toLowerCase();
  if (lowerName.startsWith("xiaomi") || lowerName.startsWith("mi") || lowerName.startsWith("mix") || lowerName.startsWith("max") || lowerName.includes('小米')) {
    return "xiaomi";
  }
  if (lowerName.startsWith("redmi") || lowerName.includes('红米')) return "redmi";
  if (lowerName.startsWith("poco")) return "poco";
  return "other";
};

const getBrandDisplayName = (brandKey) => {
  const normalizedBrandKey = String(brandKey || "other").trim().toLowerCase();
  const meta = BRAND_META[normalizedBrandKey];
  if (meta?.name) return meta.name;
  return normalizedBrandKey.charAt(0).toUpperCase() + normalizedBrandKey.slice(1);
};

const getBrandColor = (brandKey) => BRAND_META[brandKey]?.color || "#10b981";

const getBrandOrder = (brandKey) => BRAND_META[brandKey]?.order ?? 50;

const toCatalogBrandModels = (catalogItems, sourceKey) => {
  const sourceMeta = getRomDataSourceMeta(sourceKey);
  const modelsByBrand = new Map();

  (Array.isArray(catalogItems) ? catalogItems : []).forEach((item) => {
    const codename = String(item?.codename || '').trim();
    const name = String(item?.name || codename).trim();
    const sourceRef = String(item?.sourceRef || item?.seriesUrl || item?.pageUrl || codename).trim();
    const pageUrl = String(item?.pageUrl || item?.seriesUrl || '').trim();
    const brandKey = inferBrandKey(name, item?.brand);
    const hasRomCount = item?.romCount !== undefined && item?.romCount !== null;

    if (!codename || !sourceRef) {
      return;
    }

    if (!modelsByBrand.has(brandKey)) {
      modelsByBrand.set(brandKey, []);
    }

    modelsByBrand.get(brandKey).push({
      key: `${sourceKey}:${codename}`,
      sourceKey,
      sourceLabel: sourceMeta.label,
      brandKey,
      codename,
      name,
      romCount: hasRomCount ? Number(item?.romCount) || 0 : null,
      romCountLoaded: hasRomCount,
      sourceRef,
      pageUrl,
      roms: [],
      romsLoaded: false,
    });
  });

  return Array.from(modelsByBrand.entries())
    .map(([brandKey, models]) => ({
      key: `${sourceKey}:${brandKey}`,
      sourceKey,
      name: getBrandDisplayName(brandKey),
      color: getBrandColor(brandKey),
      order: getBrandOrder(brandKey),
      models: models.sort((a, b) => (
        a.name.localeCompare(b.name, 'zh-CN')
        || a.codename.localeCompare(b.codename, 'zh-CN')
      )),
    }))
    .filter((brand) => brand.models.length > 0)
    .sort((a, b) => a.order - b.order || a.name.localeCompare(b.name, 'zh-CN'))
    .map(({ order, ...brand }) => brand);
};

const sortRoms = (items) => [...items].sort((a, b) => (
  String(b.date || '').localeCompare(String(a.date || ''), 'zh-CN')
  || String(b.version || '').localeCompare(String(a.version || ''), 'zh-CN')
  || String(a.flashType || '').localeCompare(String(b.flashType || ''), 'zh-CN')
));

const getBrands = async (sourceKey = DEFAULT_ROM_DATA_SOURCE) => {
  const normalizedSourceKey = normalizeSourceKey(sourceKey);
  const sourceState = getSourceState(normalizedSourceKey);
  const sourceHandler = getSourceHandler(normalizedSourceKey);

  if (!sourceState.brandsPromise) {
    sourceState.brandsPromise = sourceHandler.loadCatalog()
      .then((catalogItems) => toCatalogBrandModels(catalogItems, normalizedSourceKey))
      .catch((error) => {
        sourceState.brandsPromise = null;
        throw error;
      });
  }

  return sourceState.brandsPromise;
};

const getModelCacheKey = (sourceKey, model) => `${normalizeSourceKey(sourceKey)}:${String(model?.sourceRef || model?.pageUrl || model?.codename || '').trim()}`;

const getRomCacheKey = (sourceKey, rom) => [
  normalizeSourceKey(sourceKey),
  String(rom?.sourceUrl || rom?.pageUrl || rom?.url || '').trim(),
  String(rom?.version || '').trim(),
  String(rom?.flashType || '').trim(),
  String(rom?.filename || '').trim(),
].join('|');

const dedupeUrls = (urls) => Array.from(new Set(
  (Array.isArray(urls) ? urls : [])
    .map((item) => String(item || '').trim())
    .filter(Boolean),
));

export const loadBrandsData = async (sourceKey = DEFAULT_ROM_DATA_SOURCE) => {
  return await getBrands(sourceKey);
};

export const ensureModelRoms = async (model, sourceKey = model?.sourceKey || DEFAULT_ROM_DATA_SOURCE) => {
  const normalizedSourceKey = normalizeSourceKey(sourceKey);
  const sourceHandler = getSourceHandler(normalizedSourceKey);
  const sourceState = getSourceState(normalizedSourceKey);
  const cacheKey = getModelCacheKey(normalizedSourceKey, model);
  if (!cacheKey) {
    return [];
  }

  if (Array.isArray(model?.roms) && model.roms.length > 0 && model.romsLoaded) {
    return model.roms;
  }

  if (sourceState.modelRomsCache.has(cacheKey)) {
    const cachedRoms = sourceState.modelRomsCache.get(cacheKey);
    model.roms = cachedRoms;
    model.romsLoaded = true;
    if (model.romCount == null) {
      model.romCount = cachedRoms.length;
      model.romCountLoaded = true;
    }
    return cachedRoms;
  }

  if (!sourceState.modelRomsPromiseMap.has(cacheKey)) {
    sourceState.modelRomsPromiseMap.set(cacheKey, (async () => {
      const romItems = await sourceHandler.loadModelRoms(model);
      const roms = sortRoms(normalizeRomList(romItems, {
        codename: model.codename,
        name: model.name,
        brand: model.brandKey,
        sourceKey: normalizedSourceKey,
        sourceLabel: getRomDataSourceMeta(normalizedSourceKey).label,
        pageUrl: model.pageUrl,
      }));
      sourceState.modelRomsCache.set(cacheKey, roms);
      model.roms = roms;
      model.romsLoaded = true;
      if (model.romCount == null) {
        model.romCount = roms.length;
        model.romCountLoaded = true;
      }
      return roms;
    })().catch((error) => {
      sourceState.modelRomsPromiseMap.delete(cacheKey);
      throw error;
    }));
  }

  const roms = await sourceState.modelRomsPromiseMap.get(cacheKey);
  sourceState.modelRomsPromiseMap.delete(cacheKey);
  return roms;
};


export const resolveRomUrlsForEntry = async (rom, sourceKey = rom?.sourceKey || DEFAULT_ROM_DATA_SOURCE) => {
  const normalizedSourceKey = normalizeSourceKey(sourceKey);
  const sourceHandler = getSourceHandler(normalizedSourceKey);
  const sourceState = getSourceState(normalizedSourceKey);
  const cacheKey = getRomCacheKey(normalizedSourceKey, rom);
  if (!cacheKey) {
    return [];
  }

  if (sourceState.romUrlCache.has(cacheKey)) {
    const cachedUrls = sourceState.romUrlCache.get(cacheKey);
    rom.urls = cachedUrls;
    rom.url = cachedUrls[0] || '';
    rom.mirrorCount = cachedUrls.length;
    return cachedUrls;
  }

  if (!sourceState.romUrlPromiseMap.has(cacheKey)) {
    sourceState.romUrlPromiseMap.set(cacheKey, sourceHandler.resolveRomUrls(rom)
      .then((urls) => dedupeUrls(urls))
      .catch((error) => {
        sourceState.romUrlPromiseMap.delete(cacheKey);
        throw error;
      }));
  }

  const resolvedUrls = await sourceState.romUrlPromiseMap.get(cacheKey);
  sourceState.romUrlPromiseMap.delete(cacheKey);

  const nextUrls = resolvedUrls.length > 0 ? resolvedUrls : dedupeUrls(rom?.urls || [rom?.url]);
  sourceState.romUrlCache.set(cacheKey, nextUrls);
  rom.urls = nextUrls;
  rom.url = nextUrls[0] || '';
  rom.mirrorCount = nextUrls.length;
  return nextUrls;
};
