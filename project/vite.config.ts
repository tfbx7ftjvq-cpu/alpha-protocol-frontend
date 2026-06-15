import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import { NodeGlobalsPolyfillPlugin } from '@esbuild-plugins/node-globals-polyfill';

// https://vitejs.dev/config/
export default defineConfig({
  publicDir: '../public',
  plugins: [react()],
  optimizeDeps: {
    esbuildOptions: {
      // 告诉 Esbuild 在预构建阶段注入 Node 全局变量
      plugins: [
        NodeGlobalsPolyfillPlugin({
          buffer: true,
          process: true,
        }),
      ],
    },
  },
  define: {
    // 确保在运行时浏览器全局 window 上能读到 global 变量
    global: 'window',
  },
});