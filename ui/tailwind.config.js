/** @type {import('tailwindcss').Config} */
export default {
  content: ['./index.html', './src/**/*.{svelte,js,ts}'],
  theme: {
    extend: {
      colors: {
        navy: {
          950: '#0a1628',
          900: '#0f1f3a',
          800: '#1a2d4d',
        },
        cyan: {
          400: '#00d9ff',
          500: '#00b8d9',
        },
      },
      fontFamily: {
        mono: ['JetBrains Mono', 'monospace'],
        sans: ['IBM Plex Sans', 'system-ui', 'sans-serif'],
      },
      backgroundImage: {
        'grid-pattern': 'linear-gradient(rgba(0, 217, 255, 0.05) 1px, transparent 1px), linear-gradient(90deg, rgba(0, 217, 255, 0.05) 1px, transparent 1px)',
      },
      backgroundSize: {
        'grid': '20px 20px',
      },
    },
  },
  plugins: [],
};
