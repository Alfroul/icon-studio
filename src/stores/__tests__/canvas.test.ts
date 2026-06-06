import { describe, it, expect, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useCanvasStore } from '../canvas'

describe('useCanvasStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('has correct defaults', () => {
    const store = useCanvasStore()
    expect(store.width).toBe(512)
    expect(store.height).toBe(512)
    expect(store.background).toBe('#FFFFFF')
    expect(store.cornerRadius).toBe(0)
    expect(store.backgroundGradient).toBeNull()
  })

  describe('syncFromResult', () => {
    it('syncs all canvas properties from result', () => {
      const store = useCanvasStore()
      store.syncFromResult({
        width: 1024,
        height: 1024,
        background: '#1a1a2e',
        corner_radius: 20,
        background_gradient: {
          type: 'linear',
          colors: ['#667eea', '#764ba2'],
          angle: 135,
        },
      })
      expect(store.width).toBe(1024)
      expect(store.height).toBe(1024)
      expect(store.background).toBe('#1a1a2e')
      expect(store.cornerRadius).toBe(20)
      expect(store.backgroundGradient).toEqual({
        type: 'linear',
        colors: ['#667eea', '#764ba2'],
        angle: 135,
      })
    })

    it('handles null gradient in result', () => {
      const store = useCanvasStore()
      store.syncFromResult({
        width: 512,
        height: 512,
        background: '#000000',
        corner_radius: 0,
      })
      expect(store.backgroundGradient).toBeNull()
    })

    it('overwrites previous values', () => {
      const store = useCanvasStore()
      store.syncFromResult({
        width: 256,
        height: 256,
        background: 'red',
        corner_radius: 10,
        background_gradient: { type: 'radial', colors: ['#fff', '#000'], angle: 0 },
      })
      expect(store.width).toBe(256)
      store.syncFromResult({
        width: 512,
        height: 512,
        background: '#FFFFFF',
        corner_radius: 0,
      })
      expect(store.width).toBe(512)
      expect(store.backgroundGradient).toBeNull()
    })
  })
})
