// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  compatibilityDate: '2024-11-01',
  devtools: { enabled: true },
  modules: ['@nuxt/eslint', '@nuxt/fonts', '@nuxt/icon', '@unocss/nuxt'],
  app: {
    head: {
      title: 'Est',
      htmlAttrs: {
        lang: 'en',
      },
      link: [
        { rel: 'icon', type: 'image/svg+xml', href: '/favicon.svg' },
      ]
    }
  },

  devServer: {
    port: 4321,
  },

  nitro: {
    routeRules: {
      '/search': { redirect: `${process.env.EST_SERVER_URL ?? "http://localhost:3000"}/search` },
    }
  },

  fonts: {
    families: [
      { name: 'Manrope', provider: 'google' },
      { name: 'JetBrains Mono', provider: 'google' },
    ]
  }
})