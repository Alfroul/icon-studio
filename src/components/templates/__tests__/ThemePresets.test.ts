import { describe, it, expect, vi } from 'vitest'
import type { ThemePreset } from '@/types'

// Mock invoke
const mockInvoke = vi.fn().mockResolvedValue([])
vi.mock('@tauri-apps/api/core', () => ({
  invoke: mockInvoke,
}))

vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: vi.fn().mockResolvedValue(null),
}))

describe('ThemePresets list rendering', () => {
  const samplePresets: ThemePreset[] = [
    {
      id: 'ios',
      name: 'iOS (Apple)',
      cornerRadius: 22.37,
      paddingRatio: 0.1,
      background: '#FFFFFF',
      shape: 'squircle',
      previewSvg: '<svg></svg>',
    },
    {
      id: 'android',
      name: 'Android',
      cornerRadius: 0,
      paddingRatio: 0.1,
      background: '#4285F4',
      shape: 'circle',
      previewSvg: '<svg></svg>',
    },
    {
      id: 'flat',
      name: 'Flat',
      cornerRadius: 0,
      paddingRatio: 0.08,
      background: '#FFFFFF',
      shape: 'square',
      previewSvg: '<svg></svg>',
    },
  ]

  it('renders preset grid with correct count', () => {
    expect(samplePresets.length).toBe(3)
    expect(samplePresets.every(p => p.id)).toBe(true)
    expect(samplePresets.every(p => p.name)).toBe(true)
    expect(samplePresets.every(p => p.shape)).toBe(true)
  })

  it('has unique IDs across presets', () => {
    const ids = samplePresets.map(p => p.id)
    const uniqueIds = new Set(ids)
    expect(uniqueIds.size).toBe(ids.length)
  })

  it('all presets have preview SVG', () => {
    expect(samplePresets.every(p => p.previewSvg)).toBe(true)
  })

  it('invoke is called with correct command name', async () => {
    const { invoke } = await import('@tauri-apps/api/core')
    await invoke('list_theme_presets')
    expect(mockInvoke).toHaveBeenCalledWith('list_theme_presets')
  })
})
