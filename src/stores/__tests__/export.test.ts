import { describe, it, expect, beforeEach, vi } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useExportStore } from '../export'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: vi.fn().mockResolvedValue(null),
}))

vi.mock('@/utils/logger', () => ({
  logError: vi.fn(),
}))

import { invoke } from '@tauri-apps/api/core'

describe('useExportStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.mocked(invoke).mockReset()
  })

  it('has correct defaults', () => {
    const store = useExportStore()
    expect(store.exporting).toBe(false)
    expect(store.outputDir).toBe('')
    expect(store.selectedFormats).toEqual(['svg', 'png', 'ico'])
    expect(store.selectedPngSizes).toEqual([16, 32, 64, 128, 256, 512])
    expect(store.exportResults).toEqual([])
    expect(store.error).toBeNull()
  })

  describe('exportSvg', () => {
    it('returns empty array when no output dir', async () => {
      const store = useExportStore()
      const result = await store.exportSvg()
      expect(result).toEqual([])
      expect(invoke).not.toHaveBeenCalled()
    })

    it('exports svg to output dir', async () => {
      vi.mocked(invoke).mockResolvedValue('/out/icon.svg')

      const store = useExportStore()
      store.outputDir = '/out'
      const result = await store.exportSvg()

      expect(invoke).toHaveBeenCalledWith('export_svg', { path: '/out/icon.svg' })
      expect(result).toEqual(['/out/icon.svg'])
    })
  })

  describe('exportPng', () => {
    it('returns empty array when no output dir', async () => {
      const store = useExportStore()
      const result = await store.exportPng()
      expect(result).toEqual([])
    })

    it('uses default sizes when none provided', async () => {
      vi.mocked(invoke).mockResolvedValue(['/out/icon-16.png', '/out/icon-32.png'])

      const store = useExportStore()
      store.outputDir = '/out'
      await store.exportPng()

      expect(invoke).toHaveBeenCalledWith('export_png', {
        sizes: [16, 32, 64, 128, 256, 512],
        outputDir: '/out',
      })
    })

    it('uses custom sizes when provided', async () => {
      vi.mocked(invoke).mockResolvedValue(['/out/icon-64.png'])

      const store = useExportStore()
      store.outputDir = '/out'
      await store.exportPng([64])

      expect(invoke).toHaveBeenCalledWith('export_png', {
        sizes: [64],
        outputDir: '/out',
      })
    })
  })

  describe('exportIco', () => {
    it('returns empty array when no output dir', async () => {
      const store = useExportStore()
      const result = await store.exportIco()
      expect(result).toEqual([])
    })

    it('uses default ico sizes', async () => {
      vi.mocked(invoke).mockResolvedValue('/out/icon.ico')

      const store = useExportStore()
      store.outputDir = '/out'
      await store.exportIco()

      expect(invoke).toHaveBeenCalledWith('export_ico', {
        sizes: [16, 32, 48, 256],
        path: '/out/icon.ico',
      })
    })
  })

  describe('exportWebp', () => {
    it('returns empty array when no output dir', async () => {
      const store = useExportStore()
      const result = await store.exportWebp()
      expect(result).toEqual([])
    })

    it('exports webp with default size 512', async () => {
      vi.mocked(invoke).mockResolvedValue(undefined)

      const store = useExportStore()
      store.outputDir = '/out'
      const result = await store.exportWebp()

      expect(invoke).toHaveBeenCalledWith('export_webp', {
        size: 512,
        path: '/out/icon.webp',
      })
      expect(result).toEqual(['/out/icon.webp'])
    })

    it('exports webp with custom size', async () => {
      vi.mocked(invoke).mockResolvedValue(undefined)

      const store = useExportStore()
      store.outputDir = '/out'
      const result = await store.exportWebp(256)

      expect(invoke).toHaveBeenCalledWith('export_webp', {
        size: 256,
        path: '/out/icon.webp',
      })
    })
  })

  describe('exportAndroidIcons', () => {
    it('returns empty array when no output dir', async () => {
      const store = useExportStore()
      const result = await store.exportAndroidIcons()
      expect(result).toEqual([])
    })

    it('exports android icons', async () => {
      vi.mocked(invoke).mockResolvedValue(['/out/mipmap-hdpi/ic_launcher.png'])

      const store = useExportStore()
      store.outputDir = '/out'
      const result = await store.exportAndroidIcons()

      expect(invoke).toHaveBeenCalledWith('export_android_icons', { outputDir: '/out' })
      expect(result).toHaveLength(1)
    })
  })

  describe('exportIosIcons', () => {
    it('returns empty array when no output dir', async () => {
      const store = useExportStore()
      const result = await store.exportIosIcons()
      expect(result).toEqual([])
    })

    it('exports ios icons', async () => {
      vi.mocked(invoke).mockResolvedValue(['/out/AppIcon-60x60@2x.png'])

      const store = useExportStore()
      store.outputDir = '/out'
      const result = await store.exportIosIcons()

      expect(invoke).toHaveBeenCalledWith('export_ios_icons', { outputDir: '/out' })
    })
  })

  describe('exportAll', () => {
    it('returns empty array when no output dir', async () => {
      const store = useExportStore()
      const result = await store.exportAll()
      expect(result).toEqual([])
    })

    it('exports all formats with selected settings', async () => {
      vi.mocked(invoke).mockResolvedValue(['/out/icon.svg', '/out/icon.png'])

      const store = useExportStore()
      store.outputDir = '/out'
      store.selectedFormats = ['svg', 'png']
      store.selectedPngSizes = [32, 64]
      const result = await store.exportAll()

      expect(invoke).toHaveBeenCalledWith('export_all', {
        outputDir: '/out',
        formats: ['svg', 'png'],
        pngSizes: [32, 64],
      })
    })
  })
})
