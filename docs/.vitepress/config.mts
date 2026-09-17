import { defineConfig } from 'vitepress'

export default defineConfig({
  title: 'modalx',
  description: 'Responsive, declarative, box-encapsulated terminal user interface and modal dialog engine for Rust.',
  base: '/modalx/',
  cleanUrls: true,
  lastUpdated: true,
  head: [
    ['meta', { name: 'theme-color', content: '#0f172a' }],
    ['meta', { name: 'author', content: 'Oguzhan Umutlu' }],
    ['meta', { property: 'og:type', content: 'website' }],
    ['meta', { property: 'og:title', content: 'modalx - Terminal Modal Framework for Rust' }],
    ['meta', { property: 'og:description', content: 'Responsive, declarative, box-encapsulated terminal user interface and modal dialog engine for Rust.' }],
  ],
  themeConfig: {
    siteTitle: 'modalx',
    nav: [
      { text: 'Guide', link: '/guide/introduction' },
      { text: 'Modals', link: '/modals/overview' },
      { text: 'Components', link: '/components/shortcuts' },
      { text: 'Cookbooks', link: '/cookbooks/interactive-wizard' },
      { text: 'API Reference', link: '/api/reference' },
      {
        text: 'v0.1.0',
        items: [
          { text: 'crates.io', link: 'https://crates.io/crates/modalx' },
          { text: 'docs.rs', link: 'https://docs.rs/modalx' },
          { text: 'Source Repository', link: 'https://github.com/OguzhanUmutlu/modalx' }
        ]
      }
    ],
    sidebar: [
      {
        text: 'Guide',
        collapsed: false,
        items: [
          { text: 'Introduction', link: '/guide/introduction' },
          { text: 'Getting Started', link: '/guide/getting-started' },
          { text: 'Terminal Architecture', link: '/guide/terminal-architecture' },
          { text: 'BoxFrame Engine', link: '/guide/box-frame-engine' }
        ]
      },
      {
        text: 'Modals',
        collapsed: false,
        items: [
          { text: 'Overview', link: '/modals/overview' },
          { text: 'SelectModal', link: '/modals/select-modal' },
          { text: 'FormModal', link: '/modals/form-modal' },
          { text: 'InputModal', link: '/modals/input-modal' },
          { text: 'ConfirmModal', link: '/modals/confirm-modal' },
          { text: 'InfoModal', link: '/modals/info-modal' },
          { text: 'TableModal', link: '/modals/table-modal' },
          { text: 'WaitingModal', link: '/modals/waiting-modal' }
        ]
      },
      {
        text: 'Components & Subsystems',
        collapsed: false,
        items: [
          { text: 'Shortcuts Bar', link: '/components/shortcuts' },
          { text: 'Navigation Stack', link: '/components/navigation-stack' },
          { text: 'Declarative Sections', link: '/components/sections' },
          { text: 'Text Flow & Reflow', link: '/components/text-flow' }
        ]
      },
      {
        text: 'Cookbooks',
        collapsed: false,
        items: [
          { text: 'Interactive Wizard', link: '/cookbooks/interactive-wizard' },
          { text: 'System Dashboard', link: '/cookbooks/system-dashboard' },
          { text: 'Validated Config Form', link: '/cookbooks/validated-config-form' }
        ]
      },
      {
        text: 'Reference',
        collapsed: false,
        items: [
          { text: 'API Symbol Index', link: '/api/reference' }
        ]
      }
    ],
    socialLinks: [
      { icon: 'github', link: 'https://github.com/OguzhanUmutlu/modalx' }
    ],
    search: {
      provider: 'local'
    },
    footer: {
      message: 'Released under the MIT License.',
      copyright: 'Copyright (c) 2026 Oguzhan Umutlu'
    }
  }
})
