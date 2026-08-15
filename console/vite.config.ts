import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import path from 'path';

export default defineConfig({
  plugins: [react()],
  build: {
    outDir: path.resolve(__dirname, '../rust-web-axum/static'),
    emptyOutDir: true,
  },
  server: {
    port: 3000,
    proxy: {
      '/service': {
        target: 'http://127.0.0.1:8081',
        changeOrigin: true,
      },
      '/repository': {
        target: 'http://127.0.0.1:8081',
        changeOrigin: true,
      },
      '/metrics': {
        target: 'http://127.0.0.1:8081',
        changeOrigin: true,
      },
    },
  },
});
