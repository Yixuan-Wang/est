import { defineConfig } from 'unocss'
import presetWind4 from '@unocss/preset-wind4'

export default defineConfig({
  // ...UnoCSS options
  presets: [presetWind4()],
  theme: {
    font: {
      sans: 'Manrope, sans-serif',
      mono: 'JetBrains Mono, monospace',
    },
  }
})
