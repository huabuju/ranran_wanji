/**
 * 通用防抖函数
 * @param {Function} fn - 需要防抖的目标函数
 * @param {number} delay - 延迟时间（毫秒），默认 300ms
 * @returns {Function} 包装后的防抖函数
 */
export function debounce(fn, delay = 300) {
  let timer = null;
  return function (...args) {
    clearTimeout(timer);
    timer = setTimeout(() => fn.apply(this, args), delay);
  };
}
