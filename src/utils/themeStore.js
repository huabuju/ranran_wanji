import { useColorMode } from '@vueuse/core';

export const themeMode = useColorMode({
  selector: 'html',
  attribute: 'class',
  modes: {
    dark: 'dark',
    light: '',
  },
  storageKey: 'app_theme',
  emitAuto: true, // 关键：允许 mode.value 等于 'auto'
});

export function cycleTheme() {
  if (themeMode.value === 'auto') {
    themeMode.value = 'light';
  } else if (themeMode.value === 'light') {
    themeMode.value = 'dark';
  } else {
    themeMode.value = 'auto';
  }
}
