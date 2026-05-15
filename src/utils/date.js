import dayjs from 'dayjs';
import 'dayjs/locale/zh-cn'; // 引入中文语言包
import relativeTime from 'dayjs/plugin/relativeTime';

// 配置 Day.js
dayjs.locale('zh-cn'); // 全局使用中文
dayjs.extend(relativeTime); // 开启相对时间插件 (e.g., "3小时前")

/**
 * 格式化完整的日期时间 (Format standard DateTime)
 * @param {string|number|Date} date 
 * @param {string} format 
 * @returns {string} 默认返回 YYYY-MM-DD HH:mm:ss
 */
export const formatDateTime = (date, format = 'YYYY-MM-DD HH:mm:ss') => {
  if (!date) return '-';
  return dayjs(date).format(format);
};

/**
 * 格式化精简日期 (Format standard Date)
 * @param {string|number|Date} date 
 * @param {string} format 
 * @returns {string} 默认返回 YYYY-MM-DD
 */
export const formatDate = (date, format = 'YYYY-MM-DD') => {
  if (!date) return '-';
  return dayjs(date).format(format);
};

/**
 * 获取相对时间 (Get distance from now, e.g., "3分钟前")
 * @param {string|number|Date} date 
 * @returns {string}
 */
export const getRelativeTime = (date) => {
  if (!date) return '-';
  return dayjs(date).fromNow();
};

export default dayjs;
