import { describe, it, expect, beforeEach, vi } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useIconSetStore } from '../iconSetStore'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

import { invoke } from '@tauri-apps/api/core'

describe('useIconSetStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.mocked(invoke).mockReset()
  })

  it('has correct defaults', () => {
    const store = useIconSetStore()
    expect(store.sets).toEqual([])
    expect(store.activeSetId).toBeNull()
    expect(store.activeSet).toBeNull()
    expect(store.searchResults).toEqual([])
    expect(store.consistencyReport).toBeNull()
    expect(store.loading).toBe(false)
    expect(store.error).toBeNull()
    expect(store.allTags).toEqual([])
  })

  describe('allTags', () => {
    it('returns sorted unique tags from active set', () => {
      const store = useIconSetStore()
      store.activeSet = {
        id: 's1',
        name: 'Test Set',
        description: '',
        entries: [
          { id: 'e1', name: 'icon1', tags: ['arrow', 'navigation'], project_path: '', thumbnail: '' },
          { id: 'e2', name: 'icon2', tags: ['arrow', 'social'], project_path: '', thumbnail: '' },
        ],
        created_at: '',
      }

      expect(store.allTags).toEqual(['arrow', 'navigation', 'social'])
    })

    it('returns empty when no active set', () => {
      const store = useIconSetStore()
      expect(store.allTags).toEqual([])
    })
  })

  describe('loadSets', () => {
    it('loads sets and clears error', async () => {
      const sets = [
        { id: 's1', name: 'Set 1', description: 'Test', entry_count: 5, created_at: '2024-01-01' },
      ]
      vi.mocked(invoke).mockResolvedValue(sets)

      const store = useIconSetStore()
      store.error = 'previous error'
      await store.loadSets()

      expect(invoke).toHaveBeenCalledWith('list_icon_sets')
      expect(store.sets).toEqual(sets)
      expect(store.error).toBeNull()
      expect(store.loading).toBe(false)
    })

    it('sets error on failure', async () => {
      vi.mocked(invoke).mockRejectedValue(new Error('load fail'))

      const store = useIconSetStore()
      await store.loadSets()

      expect(store.loading).toBe(false)
      expect(store.error).toBeTruthy()
    })
  })

  describe('createSet', () => {
    it('creates set and prepends to list', async () => {
      const newSet = { id: 's2', name: 'New Set', description: 'Desc', entry_count: 0, created_at: '' }
      vi.mocked(invoke).mockResolvedValue(newSet)

      const store = useIconSetStore()
      const result = await store.createSet('New Set', 'Desc')

      expect(invoke).toHaveBeenCalledWith('create_icon_set', { name: 'New Set', description: 'Desc' })
      expect(result).toEqual(newSet)
      expect(store.sets[0]).toEqual(newSet)
    })

    it('returns null on error', async () => {
      vi.mocked(invoke).mockRejectedValue(new Error('fail'))

      const store = useIconSetStore()
      const result = await store.createSet('Fail')

      expect(result).toBeNull()
      expect(store.error).toBeTruthy()
    })

    it('uses empty string for missing description', async () => {
      const newSet = { id: 's3', name: 'NoDesc', description: '', entry_count: 0, created_at: '' }
      vi.mocked(invoke).mockResolvedValue(newSet)

      const store = useIconSetStore()
      await store.createSet('NoDesc')

      expect(invoke).toHaveBeenCalledWith('create_icon_set', { name: 'NoDesc', description: '' })
    })
  })

  describe('selectSet', () => {
    it('loads set details and clears report', async () => {
      const fullSet = {
        id: 's1', name: 'Set 1', description: '', entries: [], created_at: '',
      }
      vi.mocked(invoke).mockResolvedValue(fullSet)

      const store = useIconSetStore()
      store.consistencyReport = { consistent: true, issues: [], summary: '' }
      await store.selectSet('s1')

      expect(invoke).toHaveBeenCalledWith('get_icon_set', { setId: 's1' })
      expect(store.activeSetId).toBe('s1')
      expect(store.activeSet).toEqual(fullSet)
      expect(store.consistencyReport).toBeNull()
    })

    it('clears active set on error', async () => {
      vi.mocked(invoke).mockRejectedValue(new Error('fail'))

      const store = useIconSetStore()
      store.activeSetId = 's1'
      await store.selectSet('s1')

      expect(store.activeSet).toBeNull()
      expect(store.error).toBeTruthy()
    })
  })

  describe('addCurrentToSet', () => {
    it('returns null when no active set', async () => {
      const store = useIconSetStore()
      const result = await store.addCurrentToSet('icon')

      expect(result).toBeNull()
      expect(invoke).not.toHaveBeenCalled()
    })

    it('adds entry and refreshes', async () => {
      const entry = { id: 'e1', name: 'icon1', tags: ['test'], project_path: '', thumbnail: '' }
      vi.mocked(invoke)
        .mockResolvedValueOnce(entry) // add_to_icon_set
        .mockResolvedValueOnce({ id: 's1', name: 'Set 1', description: '', entries: [entry], created_at: '' }) // get_icon_set
        .mockResolvedValueOnce([]) // list_icon_sets

      const store = useIconSetStore()
      store.activeSetId = 's1'
      const result = await store.addCurrentToSet('icon1', ['test'])

      expect(invoke).toHaveBeenCalledWith('add_to_icon_set', {
        setId: 's1', name: 'icon1', tags: ['test'],
      })
      expect(result).toEqual(entry)
    })
  })

  describe('removeFromSet', () => {
    it('does nothing when no active set', async () => {
      const store = useIconSetStore()
      await store.removeFromSet('e1')
      expect(invoke).not.toHaveBeenCalled()
    })

    it('removes and refreshes', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce(undefined) // remove_from_icon_set
        .mockResolvedValueOnce({ id: 's1', name: 'Set 1', description: '', entries: [], created_at: '' }) // get_icon_set
        .mockResolvedValueOnce([]) // list_icon_sets

      const store = useIconSetStore()
      store.activeSetId = 's1'
      await store.removeFromSet('e1')

      expect(invoke).toHaveBeenCalledWith('remove_from_icon_set', {
        setId: 's1', entryId: 'e1',
      })
    })
  })

  describe('exportSet', () => {
    it('returns empty when no active set', async () => {
      const store = useIconSetStore()
      const result = await store.exportSet('png', [32], '/out')

      expect(result).toEqual([])
    })

    it('exports with default params', async () => {
      vi.mocked(invoke).mockResolvedValue(['/out/icon-32.png'])

      const store = useIconSetStore()
      store.activeSetId = 's1'
      const result = await store.exportSet('png', [32], '/out')

      expect(invoke).toHaveBeenCalledWith('export_icon_set', {
        setId: 's1', format: 'png', sizes: [32], outputDir: '/out',
      })
    })
  })

  describe('checkConsistency', () => {
    it('does nothing when no active set', async () => {
      const store = useIconSetStore()
      await store.checkConsistency()
      expect(invoke).not.toHaveBeenCalled()
    })

    it('loads consistency report', async () => {
      const report = { consistent: true, issues: [], summary: 'All good' }
      vi.mocked(invoke).mockResolvedValue(report)

      const store = useIconSetStore()
      store.activeSetId = 's1'
      await store.checkConsistency()

      expect(invoke).toHaveBeenCalledWith('check_icon_set_consistency', { setId: 's1' })
      expect(store.consistencyReport).toEqual(report)
    })
  })

  describe('tagEntry', () => {
    it('does nothing when no active set', async () => {
      const store = useIconSetStore()
      await store.tagEntry('e1', ['tag'])
      expect(invoke).not.toHaveBeenCalled()
    })

    it('tags entry and refreshes', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce(undefined) // tag_icon_entry
        .mockResolvedValueOnce({ id: 's1', name: 'Set 1', description: '', entries: [], created_at: '' }) // get_icon_set

      const store = useIconSetStore()
      store.activeSetId = 's1'
      await store.tagEntry('e1', ['arrow', 'nav'])

      expect(invoke).toHaveBeenCalledWith('tag_icon_entry', {
        setId: 's1', entryId: 'e1', tags: ['arrow', 'nav'],
      })
    })
  })

  describe('search', () => {
    it('stores search results', async () => {
      const results = [
        { id: 'e1', name: 'arrow-up', tags: ['arrow'], project_path: '', thumbnail: '' },
      ]
      vi.mocked(invoke).mockResolvedValue(results)

      const store = useIconSetStore()
      await store.search('arrow')

      expect(invoke).toHaveBeenCalledWith('search_icons', {
        query: 'arrow', setId: undefined, tags: undefined,
      })
      expect(store.searchResults).toEqual(results)
    })

    it('passes setId and tags when available', async () => {
      vi.mocked(invoke).mockResolvedValue([])

      const store = useIconSetStore()
      store.activeSetId = 's1'
      await store.search('icon', ['social'])

      expect(invoke).toHaveBeenCalledWith('search_icons', {
        query: 'icon', setId: 's1', tags: ['social'],
      })
    })

    it('clears results on error', async () => {
      vi.mocked(invoke).mockRejectedValue(new Error('fail'))

      const store = useIconSetStore()
      store.searchResults = [{ id: 'e1', name: 'old', tags: [], project_path: '', thumbnail: '' }]
      await store.search('fail')

      expect(store.searchResults).toEqual([])
      expect(store.error).toBeTruthy()
    })
  })

  describe('clearSelection', () => {
    it('resets all selection state', () => {
      const store = useIconSetStore()
      store.activeSetId = 's1'
      store.activeSet = { id: 's1', name: 'Test', description: '', entries: [], created_at: '' }
      store.consistencyReport = { consistent: true, issues: [], summary: '' }

      store.clearSelection()

      expect(store.activeSetId).toBeNull()
      expect(store.activeSet).toBeNull()
      expect(store.consistencyReport).toBeNull()
    })
  })
})
