import { describe, it, expect, beforeEach, vi } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { usePagesStore } from '../pages'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

vi.mock('@/utils/logger', () => ({
  logError: vi.fn(),
}))

import { invoke } from '@tauri-apps/api/core'

describe('usePagesStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.mocked(invoke).mockReset()
  })

  it('has correct defaults', () => {
    const store = usePagesStore()
    expect(store.pages).toEqual([])
    expect(store.activePageIndex).toBe(0)
    expect(store.activePage).toBeNull()
    expect(store.hasMultiplePages).toBe(false)
  })

  describe('activePage', () => {
    it('returns null when no pages', () => {
      const store = usePagesStore()
      expect(store.activePage).toBeNull()
    })

    it('returns the active page', () => {
      const store = usePagesStore()
      const page = { id: 'p1', name: 'Page 1', width: 512, height: 512, element_count: 0, active: true }
      store.pages = [page]
      store.activePageIndex = 0
      expect(store.activePage).toEqual(page)
    })
  })

  describe('hasMultiplePages', () => {
    it('returns false with single page', () => {
      const store = usePagesStore()
      store.pages = [{ id: 'p1', name: 'Page 1', width: 512, height: 512, element_count: 0, active: true }]
      expect(store.hasMultiplePages).toBe(false)
    })

    it('returns true with multiple pages', () => {
      const store = usePagesStore()
      store.pages = [
        { id: 'p1', name: 'Page 1', width: 512, height: 512, element_count: 0, active: true },
        { id: 'p2', name: 'Page 2', width: 1024, height: 1024, element_count: 0, active: false },
      ]
      expect(store.hasMultiplePages).toBe(true)
    })
  })

  describe('refreshPages', () => {
    it('loads pages and active index', async () => {
      const pages = [
        { id: 'p1', name: 'Page 1', width: 512, height: 512, element_count: 2, active: true },
      ]
      vi.mocked(invoke)
        .mockResolvedValueOnce(pages) // list_pages
        .mockResolvedValueOnce(0) // get_active_page

      const store = usePagesStore()
      await store.refreshPages()

      expect(invoke).toHaveBeenCalledWith('list_pages')
      expect(invoke).toHaveBeenCalledWith('get_active_page')
      expect(store.pages).toEqual(pages)
      expect(store.activePageIndex).toBe(0)
    })

    it('handles error gracefully', async () => {
      vi.mocked(invoke).mockRejectedValue(new Error('fail'))

      const store = usePagesStore()
      await store.refreshPages()

      expect(store.pages).toEqual([])
    })
  })

  describe('addPage', () => {
    it('creates page and refreshes', async () => {
      const newPage = { id: 'p2', name: 'Dark', width: 512, height: 512, element_count: 0, active: true }
      vi.mocked(invoke)
        .mockResolvedValueOnce(newPage) // add_page
        .mockResolvedValueOnce([newPage]) // list_pages
        .mockResolvedValueOnce(1) // get_active_page
        .mockResolvedValueOnce({ width: 512, height: 512, background: '#FFFFFF', corner_radius: 0 }) // get_canvas
        .mockResolvedValueOnce([]) // list_elements

      const store = usePagesStore()
      const result = await store.addPage('Dark')

      expect(invoke).toHaveBeenCalledWith('add_page', { name: 'Dark', width: undefined, height: undefined })
      expect(result).toEqual(newPage)
    })

    it('passes width and height', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce({ id: 'p3', name: 'Large', width: 1024, height: 1024, element_count: 0, active: false })
        .mockResolvedValueOnce([])
        .mockResolvedValueOnce(0)
        .mockResolvedValueOnce({ width: 1024, height: 1024, background: '#FFFFFF', corner_radius: 0 })
        .mockResolvedValueOnce([])

      const store = usePagesStore()
      await store.addPage('Large', 1024, 1024)

      expect(invoke).toHaveBeenCalledWith('add_page', { name: 'Large', width: 1024, height: 1024 })
    })
  })

  describe('switchPage', () => {
    it('switches and refreshes all state', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce(undefined) // switch_page
        .mockResolvedValueOnce([]) // list_pages
        .mockResolvedValueOnce(1) // get_active_page
        .mockResolvedValueOnce({ width: 512, height: 512, background: '#FFFFFF', corner_radius: 0 }) // get_canvas
        .mockResolvedValueOnce([]) // list_elements

      const store = usePagesStore()
      await store.switchPage('p2')

      expect(invoke).toHaveBeenCalledWith('switch_page', { pageId: 'p2' })
    })
  })

  describe('deletePage', () => {
    it('deletes and refreshes', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce(undefined) // delete_page
        .mockResolvedValueOnce([]) // list_pages
        .mockResolvedValueOnce(0) // get_active_page
        .mockResolvedValueOnce({ width: 512, height: 512, background: '#FFFFFF', corner_radius: 0 }) // get_canvas
        .mockResolvedValueOnce([]) // list_elements

      const store = usePagesStore()
      await store.deletePage('p1')

      expect(invoke).toHaveBeenCalledWith('delete_page', { pageId: 'p1' })
    })
  })

  describe('duplicatePage', () => {
    it('duplicates page with new name', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce(undefined) // duplicate_page
        .mockResolvedValueOnce([]) // list_pages
        .mockResolvedValueOnce(0) // get_active_page

      const store = usePagesStore()
      await store.duplicatePage('p1', 'Page 1 Copy')

      expect(invoke).toHaveBeenCalledWith('duplicate_page', { pageId: 'p1', name: 'Page 1 Copy' })
    })
  })

  describe('renamePage', () => {
    it('renames page', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce(undefined) // rename_page
        .mockResolvedValueOnce([{ id: 'p1', name: 'Renamed', width: 512, height: 512, element_count: 0, active: true }])
        .mockResolvedValueOnce(0)

      const store = usePagesStore()
      await store.renamePage('p1', 'Renamed')

      expect(invoke).toHaveBeenCalledWith('rename_page', { pageId: 'p1', name: 'Renamed' })
    })
  })
})
