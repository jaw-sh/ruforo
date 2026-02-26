import { defineConfig } from 'vite';
import { resolve } from 'path';

export default defineConfig({
  build: {
    outDir: resolve(__dirname, 'public/assets'),
    emptyOutDir: true,
    rollupOptions: {
      input: {
        chat: resolve(__dirname, 'resources/ts/chat.ts'),
        style: resolve(__dirname, 'resources/css/main.scss'),
      },
      output: {
        entryFileNames: '[name].js',
        chunkFileNames: '[name].js',
        assetFileNames: '[name][extname]',
      },
    },
  },
  css: {
    preprocessorOptions: {
      scss: {
        api: 'modern-compiler',
      },
    },
  },
});
