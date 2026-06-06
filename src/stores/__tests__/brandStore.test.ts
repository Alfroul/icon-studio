import { describe, it, expect, beforeEach, vi } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useBrandStore } from '../brandStore'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

vi.mock('@/utils/logger', () => ({
  logError: vi.fn(),
}))

import { invoke } from '@tauri-apps/api/core'

describe('useBrandStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.mocked(invoke).mockReset()
  })

  it('has correct defaults', () => {
    const store = useBrandStore()
    expect(store.kits).toEqual([])
    expect(store.loading).toBe(false)
    expect(store.guideText).toBe('')
  })

  describe('fetchKits', () => {
    it('loads kits and manages loading state', async () => {
      const kits = [
        { id: 'k1', name: 'Brand A', colors: { primary: '#FF0000' }, variant_count: 0 },
      ]
      vi.mocked(invoke).mockResolvedValue(kits)

      const store = useBrandStore()
      const loadingStates: boolean[] = []
      store.$subscribe(() => loadingStates.push(store.loading))

      await store.fetchKits()

      expect(invoke).toHaveBeenCalledWith('list_brand_kits')
      expect(store.kits).toEqual(kits)
      expect(store.loading).toBe(false)
    })

    it('sets loading to false on error', async () => {
      vi.mocked(invoke).mockRejectedValue(new Error('fail'))

      const store = useBrandStore()
      await store.fetchKits()

      expect(store.loading).toBe(false)
      expect(store.kits).toEqual([])
    })
  })

  describe('createKit', () => {
    it('creates kit with required and optional colors', async () => {
      const newKit = { id: 'k2', name: 'Brand B', colors: { primary: '#00FF00' }, variant_count: 0 }
      vi.mocked(invoke)
        .mockResolvedValueOnce(newKit) // create_brand_kit
        .mockResolvedValueOnce([newKit]) // list_brand_kits (fetchKits)

      const store = useBrandStore()
      const result = await store.createKit('Brand B', '#00FF00')

      expect(invoke).toHaveBeenCalledWith('create_brand_kit', {
        name: 'Brand B',
        primary: '#00FF00',
        secondary: null,
        accent: null,
        neutral: null,
      })
      expect(result).toEqual(newKit)
    })

    it('passes optional colors', async () => {
      const newKit = { id: 'k3', name: 'Full', colors: {}, variant_count: 0 }
      vi.mocked(invoke)
        .mockResolvedValueOnce(newKit)
        .mockResolvedValueOnce([newKit])

      const store = useBrandStore()
      await store.createKit('Full', '#111', '#222', '#333', '#444')

      expect(invoke).toHaveBeenCalledWith('create_brand_kit', {
        name: 'Full',
        primary: '#111',
        secondary: '#222',
        accent: '#333',
        neutral: '#444',
      })
    })
  })

  describe('applyKit', () => {
    it('applies kit without mode', async () => {
      vi.mocked(invoke).mockResolvedValue(undefined)

      const store = useBrandStore()
      await store.applyKit('k1')

      expect(invoke).toHaveBeenCalledWith('apply_brand', { kitId: 'k1', mode: null })
    })

    it('applies kit with mode', async () => {
      vi.mocked(invoke).mockResolvedValue(undefined)

      const store = useBrandStore()
      await store.applyKit('k1', 'replace')

      expect(invoke).toHaveBeenCalledWith('apply_brand', { kitId: 'k1', mode: 'replace' })
    })
  })

  describe('generateVariant', () => {
    it('generates variant and refreshes', async () => {
      const variant = { id: 'k1-v1', name: 'Brand A Dark', colors: {}, variant_count: 0 }
      vi.mocked(invoke)
        .mockResolvedValueOnce(variant) // generate_brand_variant
        .mockResolvedValueOnce([]) // fetchKits

      const store = useBrandStore()
      const result = await store.generateVariant('k1', 'dark')

      expect(invoke).toHaveBeenCalledWith('generate_brand_variant', {
        kitId: 'k1', variantType: 'dark',
      })
      expect(result).toEqual(variant)
    })
  })

  describe('exportGuide', () => {
    it('exports and stores guide text', async () => {
      vi.mocked(invoke).mockResolvedValue('# Brand Guide\n...')

      const store = useBrandStore()
      const text = await store.exportGuide('k1')

      expect(invoke).toHaveBeenCalledWith('export_brand_guide', { kitId: 'k1' })
      expect(text).toBe('# Brand Guide\n...')
      expect(store.guideText).toBe('# Brand Guide\n...')
    })
  })

  describe('suggest', () => {
    it('returns suggested brand kit', async () => {
      const suggested = { id: 'k-new', name: 'Tech Brand', colors: {}, variant_count: 0 }
      vi.mocked(invoke).mockResolvedValue(suggested)

      const store = useBrandStore()
      const result = await store.suggest('A modern tech startup')

      expect(invoke).toHaveBeenCalledWith('suggest_brand', { description: 'A modern tech startup' })
      expect(result).toEqual(suggested)
    })
  })

  describe('deleteKit', () => {
    it('deletes and refreshes', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce(undefined) // delete_brand_kit
        .mockResolvedValueOnce([]) // fetchKits

      const store = useBrandStore()
      await store.deleteKit('k1')

      expect(invoke).toHaveBeenCalledWith('delete_brand_kit', { kitId: 'k1' })
    })
  })

  describe('updateColor', () => {
    it('updates color and refreshes', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce(undefined) // update_brand_kit_color
        .mockResolvedValueOnce([]) // fetchKits

      const store = useBrandStore()
      await store.updateColor('k1', 'primary', '#00FF00')

      expect(invoke).toHaveBeenCalledWith('update_brand_kit_color', {
        kitId: 'k1', role: 'primary', color: '#00FF00',
      })
    })
  })
})
