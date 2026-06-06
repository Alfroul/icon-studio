import { describe, it, expect, beforeEach, vi } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useElementsStore } from '../elements'

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

function makeShape(id: string, fill = '#FF0000') {
  return {
    id,
    type: 'shape' as const,
    shape_type: 'circle' as const,
    x: 0, y: 0, width: 100, height: 100,
    fill,
    stroke: null, stroke_width: 0,
    opacity: 1, rotation: 0, border_radius: 0,
    gradient: null, shadows: [], animation: null,
    blend_mode: null, clip_element_id: null, mask_element_id: null,
    locked: false, visible: true, svg_filter: null,
  }
}

describe('useElementsStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.mocked(invoke).mockReset()
  })

  it('has correct defaults', () => {
    const store = useElementsStore()
    expect(store.items).toEqual([])
    expect(store.isDialogOpen).toBe(false)
    expect(store.currentVersion).toBe(0)
  })

  describe('refreshElements', () => {
    it('updates items and bumps version', async () => {
      const elements = [makeShape('s1'), makeShape('s2')]
      vi.mocked(invoke).mockResolvedValue(elements)

      const store = useElementsStore()
      await store.refreshElements()

      expect(invoke).toHaveBeenCalledWith('list_elements')
      expect(store.items).toEqual(elements)
      expect(store.currentVersion).toBe(1)
    })

    it('does not update items on invoke failure', async () => {
      vi.mocked(invoke).mockRejectedValue(new Error('fail'))

      const store = useElementsStore()
      await store.refreshElements()

      expect(store.items).toEqual([])
      expect(store.currentVersion).toBe(0)
    })
  })

  describe('addShape', () => {
    it('calls invoke and refreshes', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce(undefined) // add_shape
        .mockResolvedValueOnce([makeShape('s1')]) // list_elements

      const store = useElementsStore()
      await store.addShape('circle', '#FF0000', 100, 50, 50)

      expect(invoke).toHaveBeenCalledWith('add_shape', {
        shapeType: 'circle', fill: '#FF0000', size: 100, x: 50, y: 50,
      })
      expect(store.items).toHaveLength(1)
    })

    it('handles error gracefully', async () => {
      vi.mocked(invoke).mockRejectedValue(new Error('add failed'))

      const store = useElementsStore()
      await store.addShape('circle', '#FF0000', 100, 0, 0)

      expect(store.items).toEqual([])
    })
  })

  describe('addText', () => {
    it('calls invoke with text params', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce(undefined)
        .mockResolvedValueOnce([])

      const store = useElementsStore()
      await store.addText('Hello', 'Arial', 24, '#000', 10, 20)

      expect(invoke).toHaveBeenCalledWith('add_text', {
        content: 'Hello', fontFamily: 'Arial', fontSize: 24, fill: '#000', x: 10, y: 20,
      })
    })
  })

  describe('addIcon', () => {
    it('calls invoke with icon params', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce(undefined)
        .mockResolvedValueOnce([])

      const store = useElementsStore()
      await store.addIcon('heart', '#E74C3C', 64, 0, 0)

      expect(invoke).toHaveBeenCalledWith('add_icon', {
        iconName: 'heart', fill: '#E74C3C', size: 64, x: 0, y: 0,
      })
    })
  })

  describe('addPath', () => {
    it('calls invoke with path params', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce(undefined)
        .mockResolvedValueOnce([])

      const store = useElementsStore()
      await store.addPath('M 0 0 L 10 10', '#000', 2)

      expect(invoke).toHaveBeenCalledWith('add_path', {
        d: 'M 0 0 L 10 10', stroke: '#000', strokeWidth: 2, fill: null,
      })
    })

    it('passes fill when provided', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce(undefined)
        .mockResolvedValueOnce([])

      const store = useElementsStore()
      await store.addPath('M 0 0 L 10 10', '#000', 2, '#FF0000')

      expect(invoke).toHaveBeenCalledWith('add_path', {
        d: 'M 0 0 L 10 10', stroke: '#000', strokeWidth: 2, fill: '#FF0000',
      })
    })
  })

  describe('removeElement', () => {
    it('calls invoke with element id', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce(undefined)
        .mockResolvedValueOnce([])

      const store = useElementsStore()
      await store.removeElement('s1')

      expect(invoke).toHaveBeenCalledWith('remove_element', { elementId: 's1' })
    })
  })

  describe('updateElement', () => {
    it('calls set_props with element id and props', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce(undefined)
        .mockResolvedValueOnce([])

      const store = useElementsStore()
      await store.updateElement('s1', { fill: '#00FF00', opacity: 0.5 })

      expect(invoke).toHaveBeenCalledWith('set_props', {
        elementId: 's1', props: { fill: '#00FF00', opacity: 0.5 },
      })
    })
  })

  describe('reorderElement', () => {
    it('calls reorder_elements with id and index', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce(undefined)
        .mockResolvedValueOnce([])

      const store = useElementsStore()
      await store.reorderElement('s1', 2)

      expect(invoke).toHaveBeenCalledWith('reorder_elements', {
        elementId: 's1', newIndex: 2,
      })
    })
  })

  describe('duplicateElement', () => {
    it('calls duplicate and selects new element', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce('s1-copy') // duplicate_element
        .mockResolvedValueOnce([makeShape('s1'), makeShape('s1-copy')]) // list_elements

      const store = useElementsStore()
      await store.duplicateElement('s1')

      expect(invoke).toHaveBeenCalledWith('duplicate_element', { elementId: 's1' })
      expect(store.items).toHaveLength(2)
    })
  })

  describe('setGradient / clearGradient', () => {
    it('sets gradient', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce(undefined)
        .mockResolvedValueOnce([])

      const store = useElementsStore()
      await store.setGradient('s1', 'linear', ['#FF0000', '#00FF00'], 90)

      expect(invoke).toHaveBeenCalledWith('set_gradient', {
        elementId: 's1', gradientType: 'linear', colors: ['#FF0000', '#00FF00'], angle: 90,
      })
    })

    it('clears gradient', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce(undefined)
        .mockResolvedValueOnce([])

      const store = useElementsStore()
      await store.clearGradient('s1')

      expect(invoke).toHaveBeenCalledWith('clear_gradient', { elementId: 's1' })
    })
  })

  describe('setShadow / clearShadow', () => {
    it('sets shadow', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce(undefined)
        .mockResolvedValueOnce([])

      const store = useElementsStore()
      await store.setShadow('s1', '#000000', 8, 0, 4)

      expect(invoke).toHaveBeenCalledWith('set_shadow', {
        elementId: 's1', color: '#000000', blur: 8, offsetX: 0, offsetY: 4,
      })
    })

    it('clears shadow', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce(undefined)
        .mockResolvedValueOnce([])

      const store = useElementsStore()
      await store.clearShadow('s1')

      expect(invoke).toHaveBeenCalledWith('clear_shadow', { elementId: 's1' })
    })
  })

  describe('setFilter / clearFilter', () => {
    it('sets filter', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce(undefined)
        .mockResolvedValueOnce([])

      const store = useElementsStore()
      await store.setFilter('s1', 'blur', { amount: 5 })

      expect(invoke).toHaveBeenCalledWith('set_filter', {
        elementId: 's1', filterType: 'blur', params: { amount: 5 },
      })
    })

    it('clears filter', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce(undefined)
        .mockResolvedValueOnce([])

      const store = useElementsStore()
      await store.clearFilter('s1')

      expect(invoke).toHaveBeenCalledWith('clear_filter', { elementId: 's1' })
    })
  })

  describe('setBlendMode', () => {
    it('sets blend mode', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce(undefined)
        .mockResolvedValueOnce([])

      const store = useElementsStore()
      await store.setBlendMode('s1', 'multiply')

      expect(invoke).toHaveBeenCalledWith('set_blend_mode', {
        elementId: 's1', mode: 'multiply',
      })
    })

    it('passes null for clearing blend mode', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce(undefined)
        .mockResolvedValueOnce([])

      const store = useElementsStore()
      await store.setBlendMode('s1', null)

      expect(invoke).toHaveBeenCalledWith('set_blend_mode', {
        elementId: 's1', mode: null,
      })
    })
  })

  describe('groupElements / ungroup', () => {
    it('groups elements', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce(undefined)
        .mockResolvedValueOnce([])

      const store = useElementsStore()
      await store.groupElements(['s1', 's2'])

      expect(invoke).toHaveBeenCalledWith('group_elements', { elementIds: ['s1', 's2'] })
    })

    it('ungroups elements', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce(undefined)
        .mockResolvedValueOnce([])

      const store = useElementsStore()
      await store.ungroup('g1')

      expect(invoke).toHaveBeenCalledWith('ungroup', { groupId: 'g1' })
    })
  })

  describe('addToGroup / removeFromGroup', () => {
    it('adds to group', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce(undefined)
        .mockResolvedValueOnce([])

      const store = useElementsStore()
      await store.addToGroup('g1', 's1')

      expect(invoke).toHaveBeenCalledWith('add_to_group', { groupId: 'g1', elementId: 's1' })
    })

    it('removes from group', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce(undefined)
        .mockResolvedValueOnce([])

      const store = useElementsStore()
      await store.removeFromGroup('g1', 's1')

      expect(invoke).toHaveBeenCalledWith('remove_from_group', { groupId: 'g1', elementId: 's1' })
    })
  })

  describe('setClip / clearClip / setMask / clearMask', () => {
    it('sets clip', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce(undefined)
        .mockResolvedValueOnce([])

      const store = useElementsStore()
      await store.setClip('s1', 's2')

      expect(invoke).toHaveBeenCalledWith('set_clip', { elementId: 's1', clipElementId: 's2' })
    })

    it('clears clip', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce(undefined)
        .mockResolvedValueOnce([])

      const store = useElementsStore()
      await store.clearClip('s1')

      expect(invoke).toHaveBeenCalledWith('clear_clip', { elementId: 's1' })
    })

    it('sets mask', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce(undefined)
        .mockResolvedValueOnce([])

      const store = useElementsStore()
      await store.setMask('s1', 's2')

      expect(invoke).toHaveBeenCalledWith('set_mask', { elementId: 's1', maskElementId: 's2' })
    })

    it('clears mask', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce(undefined)
        .mockResolvedValueOnce([])

      const store = useElementsStore()
      await store.clearMask('s1')

      expect(invoke).toHaveBeenCalledWith('clear_mask', { elementId: 's1' })
    })
  })

  describe('booleanOp', () => {
    it('calls boolean_operation and selects result', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce('bool-1') // boolean_operation
        .mockResolvedValueOnce([makeShape('bool-1')]) // list_elements

      const store = useElementsStore()
      await store.booleanOp('s1', 's2', 'union')

      expect(invoke).toHaveBeenCalledWith('boolean_operation', {
        elementAId: 's1', elementBId: 's2', operation: 'union',
      })
    })
  })

  describe('convertToPath', () => {
    it('calls convert_to_path and selects result', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce('path-1')
        .mockResolvedValueOnce([])

      const store = useElementsStore()
      await store.convertToPath('s1')

      expect(invoke).toHaveBeenCalledWith('convert_to_path', { elementId: 's1' })
    })
  })

  describe('setLayout', () => {
    it('calls set_layout with params', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce(undefined)
        .mockResolvedValueOnce([])

      const store = useElementsStore()
      await store.setLayout('grid', 10, 20)

      expect(invoke).toHaveBeenCalledWith('set_layout', {
        layoutType: 'grid', gap: 10, padding: 20,
      })
    })
  })

  describe('listFonts', () => {
    it('returns fonts from invoke', async () => {
      const fonts = [{ name: 'Arial', family: 'Arial', style: 'Regular', weight: 400 }]
      vi.mocked(invoke).mockResolvedValue(fonts)

      const store = useElementsStore()
      const result = await store.listFonts('Arial')

      expect(invoke).toHaveBeenCalledWith('list_fonts', { keyword: 'Arial' })
      expect(result).toEqual(fonts)
    })

    it('returns empty array on error', async () => {
      vi.mocked(invoke).mockRejectedValue(new Error('fail'))

      const store = useElementsStore()
      const result = await store.listFonts()

      expect(result).toEqual([])
    })
  })

  describe('setOnChanged', () => {
    it('calls onChanged callback after refresh', async () => {
      vi.mocked(invoke).mockResolvedValue([])

      const callback = vi.fn().mockResolvedValue(undefined)
      const store = useElementsStore()
      store.setOnChanged(callback)
      await store.refreshElements()

      expect(callback).toHaveBeenCalledTimes(1)
    })
  })
})
