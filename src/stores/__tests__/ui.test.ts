import { describe, it, expect, beforeEach, vi } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useUiStore } from '../ui'

describe('useUiStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('has correct defaults', () => {
    const store = useUiStore()
    expect(store.activePanel).toBe('canvas')
    expect(store.selectedElementId).toBeNull()
    expect(store.selectedElementIds.size).toBe(0)
    expect(store.zoom).toBe(100)
    expect(store.displayZoom).toBe(100)
    expect(store.panX).toBe(0)
    expect(store.panY).toBe(0)
    expect(store.iconBrowserOpen).toBe(false)
    expect(store.canUndo).toBe(false)
    expect(store.canRedo).toBe(false)
    expect(store.toasts).toEqual([])
    expect(store.isDrawing).toBe(false)
    expect(store.currentPath).toEqual([])
    expect(store.sidebarCollapsed).toBe(false)
  })

  describe('panel', () => {
    it('sets active panel', () => {
      const store = useUiStore()
      store.setPanel('elements')
      expect(store.activePanel).toBe('elements')
      store.setPanel('export')
      expect(store.activePanel).toBe('export')
    })
  })

  describe('element selection', () => {
    it('selects a single element', () => {
      const store = useUiStore()
      store.selectElement('shape-1')
      expect(store.selectedElementId).toBe('shape-1')
      expect(store.selectedElementIds.has('shape-1')).toBe(true)
    })

    it('deselects when null', () => {
      const store = useUiStore()
      store.selectElement('shape-1')
      store.selectElement(null)
      expect(store.selectedElementId).toBeNull()
      expect(store.selectedElementIds.size).toBe(0)
    })

    it('selects multiple elements', () => {
      const store = useUiStore()
      store.selectElements(['shape-1', 'text-2'])
      expect(store.selectedElementIds.has('shape-1')).toBe(true)
      expect(store.selectedElementIds.has('text-2')).toBe(true)
      expect(store.selectedElementId).toBeNull()
    })

    it('selects single element via selectElements', () => {
      const store = useUiStore()
      store.selectElements(['icon-3'])
      expect(store.selectedElementId).toBe('icon-3')
    })

    it('toggles element selection', () => {
      const store = useUiStore()
      store.toggleElementSelection('shape-1')
      expect(store.selectedElementIds.has('shape-1')).toBe(true)
      store.toggleElementSelection('shape-1')
      expect(store.selectedElementIds.has('shape-1')).toBe(false)
    })

    it('clears selection', () => {
      const store = useUiStore()
      store.selectElements(['shape-1', 'text-2'])
      store.clearSelection()
      expect(store.selectedElementIds.size).toBe(0)
      expect(store.selectedElementId).toBeNull()
    })
  })

  describe('zoom', () => {
    it('clamps zoom to 25-400 range', () => {
      const store = useUiStore()
      store.setZoom(10)
      expect(store.zoom).toBe(25)
      store.setZoom(500)
      expect(store.zoom).toBe(400)
    })

    it('snaps zoom to nearest 5', () => {
      const store = useUiStore()
      store.setZoom(73)
      expect(store.zoom).toBe(75)
      store.setZoom(72)
      expect(store.zoom).toBe(70)
    })

    it('syncs displayZoom with zoom', () => {
      const store = useUiStore()
      store.setZoom(200)
      expect(store.displayZoom).toBe(200)
    })
  })

  describe('pan', () => {
    it('sets pan position', () => {
      const store = useUiStore()
      store.setPan(100, 200)
      expect(store.panX).toBe(100)
      expect(store.panY).toBe(200)
    })

    it('resets view to defaults', () => {
      const store = useUiStore()
      store.setZoom(200)
      store.setPan(50, 50)
      store.resetView()
      expect(store.zoom).toBe(100)
      expect(store.displayZoom).toBe(100)
      expect(store.panX).toBe(0)
      expect(store.panY).toBe(0)
    })
  })

  describe('icon browser', () => {
    it('toggles icon browser', () => {
      const store = useUiStore()
      expect(store.iconBrowserOpen).toBe(false)
      store.toggleIconBrowser()
      expect(store.iconBrowserOpen).toBe(true)
      store.toggleIconBrowser()
      expect(store.iconBrowserOpen).toBe(false)
    })

    it('sets icon browser to specific state', () => {
      const store = useUiStore()
      store.toggleIconBrowser(true)
      expect(store.iconBrowserOpen).toBe(true)
      store.toggleIconBrowser(false)
      expect(store.iconBrowserOpen).toBe(false)
    })
  })

  describe('sidebar', () => {
    it('toggles sidebar collapsed', () => {
      const store = useUiStore()
      expect(store.sidebarCollapsed).toBe(false)
      store.toggleSidebar()
      expect(store.sidebarCollapsed).toBe(true)
      store.toggleSidebar()
      expect(store.sidebarCollapsed).toBe(false)
    })
  })

  describe('toast', () => {
    it('adds a toast message', () => {
      vi.useFakeTimers()
      const store = useUiStore()
      store.showToast('test message', 'success')
      expect(store.toasts.length).toBe(1)
      expect(store.toasts[0].text).toBe('test message')
      expect(store.toasts[0].type).toBe('success')
      vi.useRealTimers()
    })

    it('removes toast after duration', () => {
      vi.useFakeTimers()
      const store = useUiStore()
      store.showToast('will expire', 'info', 3000)
      expect(store.toasts.length).toBe(1)
      vi.advanceTimersByTime(3000)
      expect(store.toasts.length).toBe(0)
      vi.useRealTimers()
    })

    it('increments toast ids', () => {
      vi.useFakeTimers()
      const store = useUiStore()
      store.showToast('first', 'info')
      store.showToast('second', 'info')
      expect(store.toasts[0].id).toBeLessThan(store.toasts[1].id)
      vi.useRealTimers()
    })
  })
})
