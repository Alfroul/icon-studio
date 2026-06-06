import { describe, it, expect, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useSettingsStore } from '../settings'

describe('useSettingsStore', () => {
  beforeEach(() => {
    localStorage.clear()
    setActivePinia(createPinia())
  })

  it('has correct defaults', () => {
    const store = useSettingsStore()
    expect(store.theme).toBe('dark')
    expect(store.mcpEnabled).toBe(true)
    expect(store.autoExportOnClose).toBe(false)
    expect(store.autoExportDir).toBe('')
    expect(store.defaultFontFamily).toBe('sans-serif')
    expect(store.defaultExportFormats).toEqual(['svg', 'png'])
  })

  it('reads persisted theme from localStorage', () => {
    localStorage.setItem('iconstudio-theme', 'light')
    const store = useSettingsStore()
    expect(store.theme).toBe('light')
  })

  it('toggles theme via applyTheme', () => {
    const store = useSettingsStore()
    store.applyTheme('light')
    expect(store.theme).toBe('light')
    expect(localStorage.getItem('iconstudio-theme')).toBe('light')

    store.applyTheme('dark')
    expect(store.theme).toBe('dark')
    expect(localStorage.getItem('iconstudio-theme')).toBe('dark')
  })

  it('sets mcpEnabled', () => {
    const store = useSettingsStore()
    expect(store.mcpEnabled).toBe(true)
    store.setMcpEnabled(false)
    expect(store.mcpEnabled).toBe(false)
    expect(localStorage.getItem('iconstudio-mcp-enabled')).toBe('false')
  })

  it('sets defaultFont', () => {
    const store = useSettingsStore()
    store.setDefaultFont('Inter')
    expect(store.defaultFontFamily).toBe('Inter')
    expect(localStorage.getItem('iconstudio-default-font')).toBe('Inter')
  })

  it('sets defaultExportFormats', () => {
    const store = useSettingsStore()
    store.setDefaultExportFormats(['svg', 'png', 'ico'])
    expect(store.defaultExportFormats).toEqual(['svg', 'png', 'ico'])
    expect(localStorage.getItem('iconstudio-default-export-formats')).toBe(
      JSON.stringify(['svg', 'png', 'ico'])
    )
  })

  it('sets autoExportOnClose', () => {
    const store = useSettingsStore()
    expect(store.autoExportOnClose).toBe(false)
    store.setAutoExportOnClose(true)
    expect(store.autoExportOnClose).toBe(true)
    expect(localStorage.getItem('iconstudio-auto-export-on-close')).toBe('true')
  })

  it('sets autoExportDir', () => {
    const store = useSettingsStore()
    store.setAutoExportDir('/tmp/exports')
    expect(store.autoExportDir).toBe('/tmp/exports')
    expect(localStorage.getItem('iconstudio-auto-export-dir')).toBe('/tmp/exports')
  })
})
