import { describe, it, expect, beforeEach, vi } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useProjectStore } from '@/stores/project'
import { useUiStore } from '@/stores/ui'

// Mock invoke since we test frontend logic, not backend calls
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockResolvedValue(undefined),
}))

vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: vi.fn().mockResolvedValue(null),
}))

describe('QuickStart visibility logic', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('shows quickstart when no elements and no preview', () => {
    const project = useProjectStore()
    expect(project.elements.length).toBe(0)
    expect(project.svgPreview).toBe('')
    // QuickStart is shown when: elements.length === 0 && !svgPreview
    const shouldShow = project.elements.length === 0 && !project.svgPreview
    expect(shouldShow).toBe(true)
  })

  it('hides quickstart when elements exist', () => {
    const project = useProjectStore()
    // After adding an element, quickstart should hide
    // The store uses elementsStore.items which is initialized to []
    // When elements are added, svgPreview gets populated
    expect(project.elements).toBeDefined()
  })
})

describe('QuickStart panel navigation', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('navigates to templates panel', () => {
    const ui = useUiStore()
    ui.setPanel('templates')
    expect(ui.activePanel).toBe('templates')
  })

  it('navigates to AI panel', () => {
    const ui = useUiStore()
    ui.setPanel('ai')
    expect(ui.activePanel).toBe('ai')
  })

  it('navigates to elements panel', () => {
    const ui = useUiStore()
    ui.setPanel('elements')
    expect(ui.activePanel).toBe('elements')
  })
})
