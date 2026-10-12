import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
  build: {
    target: 'esnext',
  },
  server: {
    port: 3000,
    host: '0.0.0.0',
    proxy: {
      '/api': {
        target: 'http://127.0.0.1:8080',
        changeOrigin: true,
        ws: true,
      },
      '/swagger-ui': {
        target: 'http://127.0.0.1:8080',
        changeOrigin: true,
      },
      '/api-docs': {
        target: 'http://127.0.0.1:8080',
        changeOrigin: true,
      },
      '/novnc': {
        target: 'http://127.0.0.1:8080',
        changeOrigin: true,
      },
    },
  },
});
