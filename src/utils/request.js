import axios from 'axios';
import { ElMessage } from 'element-plus';

// 创建 axios 实例 (Create Axios Instance)
// 这里配置基础路径和超时时间。后续若需要接入环境变量可直接替换 baseURL。
const service = axios.create({
  baseURL: import.meta.env.VITE_API_BASE_URL || '/api', // Default API path
  timeout: 10000, // Request timeout (10 seconds)
  headers: {
    'Content-Type': 'application/json;charset=utf-8'
  }
});

// 请求拦截器 (Request Interceptor)
service.interceptors.request.use(
  (config) => {
    // 可以在这里统一处理 Token 注入等逻辑 (e.g., config.headers.Authorization = `Bearer ${token}`)
    
    // 仅用于控制台调试 (Debug log)
    if (import.meta.env.DEV) {
      console.log(`[HTTP Request] -> ${config.method?.toUpperCase()} ${config.url}`);
    }
    
    return config;
  },
  (error) => {
    // 请求失败的统一处理
    console.error('Request Error:', error);
    return Promise.reject(error);
  }
);

// 响应拦截器 (Response Interceptor)
service.interceptors.response.use(
  (response) => {
    // 自动解包 Axios 的 data 层。视后端返回的数据结构而定，如果后端统一返回 { code, data, msg } 结构，
    // 可以在这里做全局业务状态码拦截 (Global Business Error Catching)
    
    const res = response.data;
    
    // 假设业务上规定 code !== 200 就是报错 (假定后端规范)
    // if (res.code && res.code !== 200) {
    //   ElMessage.error(res.msg || '操作失败 (Operation Failed)');
    //   return Promise.reject(new Error(res.msg || 'Error'));
    // }
    
    return res;
  },
  (error) => {
    // 处理 HTTP 网络层的错误 (Standard HTTP Errors: 401, 403, 404, 500 etc.)
    let message = '网络异常 (Network Error)';
    
    if (error.response) {
      switch (error.response.status) {
        case 401:
          message = '未授权，请重新登录 (Unauthorized)';
          // 可在此处触发登出逻辑 (Logout Logic)
          break;
        case 403:
          message = '拒绝访问 (Forbidden)';
          break;
        case 404:
          message = '请求地址错误 (Not Found)';
          break;
        case 500:
          message = '服务器内部错误 (Internal Server Error)';
          break;
        default:
          message = `系统异常: ${error.response.status}`;
      }
    } else if (error.message && error.message.includes('timeout')) {
      message = '请求超时 (Request Timeout)';
    }

    ElMessage.error(message);
    return Promise.reject(error);
  }
);

export default service;
