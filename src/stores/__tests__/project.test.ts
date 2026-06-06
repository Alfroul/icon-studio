import { describe, it, expect, beforeEach, vi } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useProjectStore } from '../project'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: vi.fn().mockResolvedValue(null),
}))

vi.mock('@/utils/logger', () => ({
  logError: vi.fn(),
  logWarn: vi.fn(),
}))

import { invoke } from '@tauri-apps/api/core'

describe('useProjectStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.mocked(invoke).mockReset()
  })

  it('has correct defaults', () => {
    const store = useProjectStore()
    expect(store.svgPreview).toBe('')
    expect(store.elements).toEqual([])
    expect(store.isDialogOpen).toBe(false)
    expect(store.selectedElement).toBeNull()
  })

  describe('fetchPreview', () => {
    it('updates svgPreview', async () => {
      vi.mocked(invoke).mockResolvedValue('<svg>test</svg>')

      const store = useProjectStore()
      await store.fetchPreview()

      expect(invoke).toHaveBeenCalledWith('render_preview')
      expect(store.svgPreview).toBe('<svg>test</svg>')
    })

    it('handles error gracefully', async () => {
      vi.mocked(invoke).mockRejectedValue(new Error('render fail'))

      const store = useProjectStore()
      await store.fetchPreview()

      expect(store.svgPreview).toBe('')
    })
  })

  describe('fetchStatus', () => {
    it('returns status string', async () => {
      vi.mocked(invoke).mockResolvedValue('ready')

      const store = useProjectStore()
      const status = await store.fetchStatus()

      expect(invoke).toHaveBeenCalledWith('get_status')
      expect(status).toBe('ready')
    })

    it('returns empty string on error', async () => {
      vi.mocked(invoke).mockRejectedValue(new Error('fail'))

      const store = useProjectStore()
      const status = await store.fetchStatus()

      expect(status).toBe('')
    })
  })

  describe('newProject', () => {
    it('creates new project and resets state', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce({ width: 512, height: 512, background: '#FFFFFF', corner_radius: 0 }) // new_canvas
        .mockResolvedValueOnce(true) // can_undo
        .mockResolvedValueOnce(true) // can_redo
        .mockResolvedValueOnce('<svg>new</svg>') // render_preview

      const store = useProjectStore()
      await store.newProject()

      expect(invoke).toHaveBeenCalledWith('new_canvas')
      expect(store.svgPreview).toBe('<svg>new</svg>')
      expect(store.elements).toEqual([])
    })

    it('handles error gracefully', async () => {
      vi.mocked(invoke).mockRejectedValue(new Error('create fail'))

      const store = useProjectStore()
      await store.newProject()

      expect(store.elements).toEqual([])
    })
  })

  describe('saveProject', () => {
    it('saves with explicit path', async () => {
      vi.mocked(invoke).mockResolvedValue('/path/to/file.iconproject.json')

      const store = useProjectStore()
      await store.saveProject('/path/to/file.iconproject.json')

      expect(invoke).toHaveBeenCalledWith('save_project', { path: '/path/to/file.iconproject.json' })
    })

    it('saves with null path', async () => {
      vi.mocked(invoke).mockResolvedValue('/default/path.iconproject.json')

      const store = useProjectStore()
      await store.saveProject()

      expect(invoke).toHaveBeenCalledWith('save_project', { path: null })
    })
  })

  describe('performUndo', () => {
    it('refreshes state after successful undo', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce(true) // undo
        .mockResolvedValueOnce(true) // can_undo
        .mockResolvedValueOnce(false) // can_redo
        .mockResolvedValueOnce([]) // list_elements
        .mockResolvedValueOnce({ width: 512, height: 512, background: '#FFFFFF', corner_radius: 0 }) // get_canvas

      const store = useProjectStore()
      await store.performUndo()

      expect(invoke).toHaveBeenCalledWith('undo')
      expect(invoke).toHaveBeenCalledWith('can_undo')
      expect(invoke).toHaveBeenCalledWith('can_redo')
    })

    it('does nothing when undo returns false', async () => {
      vi.mocked(invoke).mockResolvedValue(false)

      const store = useProjectStore()
      await store.performUndo()

      expect(invoke).toHaveBeenCalledWith('undo')
      expect(invoke).toHaveBeenCalledTimes(1)
    })
  })

  describe('performRedo', () => {
    it('refreshes state after successful redo', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce(true) // redo
        .mockResolvedValueOnce(false) // can_undo
        .mockResolvedValueOnce(true) // can_redo
        .mockResolvedValueOnce([]) // list_elements
        .mockResolvedValueOnce({ width: 512, height: 512, background: '#FFFFFF', corner_radius: 0 }) // get_canvas

      const store = useProjectStore()
      await store.performRedo()

      expect(invoke).toHaveBeenCalledWith('redo')
    })

    it('does nothing when redo returns false', async () => {
      vi.mocked(invoke).mockResolvedValue(false)

      const store = useProjectStore()
      await store.performRedo()

      expect(invoke).toHaveBeenCalledTimes(1)
    })
  })

  describe('selectedElement', () => {
    it('returns null when no element selected', () => {
      const store = useProjectStore()
      expect(store.selectedElement).toBeNull()
    })

    it('returns element matching selected id', async () => {
      const shape = {
        id: 's1', type: 'shape' as const, shape_type: 'circle' as const,
        x: 0, y: 0, width: 100, height: 100, fill: '#FF0000',
        stroke: null, stroke_width: 0, opacity: 1, rotation: 0, border_radius: 0,
        gradient: null, shadows: [], animation: null,
        blend_mode: null, clip_element_id: null, mask_element_id: null,
        locked: false, visible: true, svg_filter: null,
      }
      vi.mocked(invoke).mockResolvedValue([shape])

      const store = useProjectStore()
      await store.refreshElements()

      const ui = await import('../ui').then(m => m.useUiStore())
      ui.selectElement('s1')

      expect(store.selectedElement).toEqual(shape)
    })
  })
})
