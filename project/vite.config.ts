import { defineConfig } from 'vite';
import type { Plugin } from 'vite';
import { mkdirSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import react from '@vitejs/plugin-react';
import { NodeGlobalsPolyfillPlugin } from '@esbuild-plugins/node-globals-polyfill';
import { resolveReleaseManifest } from './scripts/release-manifest.ts';

function releaseManifestPlugin(): Plugin {
  return {
    name: 'alpha-release-manifest',
    apply: 'build',
    closeBundle() {
      const distDirectory = resolve(process.cwd(), 'dist');
      mkdirSync(distDirectory, { recursive: true });
      writeFileSync(
        resolve(distDirectory, 'release.json'),
        `${JSON.stringify(resolveReleaseManifest())}\n`,
        'utf8',
      );
    },
  };
}

// https://vitejs.dev/config/
export default defineConfig({
  publicDir: '../public',
  plugins: [react(), releaseManifestPlugin()],
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
