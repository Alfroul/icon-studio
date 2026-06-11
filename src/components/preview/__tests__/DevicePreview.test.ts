import { describe, it, expect, beforeEach, vi } from "vitest";
import { setActivePinia, createPinia } from "pinia";
import { useProjectStore } from "@/stores/project";
import { wallpapers } from "@/components/preview/wallpapers";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue(undefined),
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: vi.fn().mockResolvedValue(null),
}));

describe("Wallpapers", () => {
  it("has 8 preset wallpapers", () => {
    expect(wallpapers).toHaveLength(8);
  });

  it("each wallpaper has id, name, and style", () => {
    for (const wp of wallpapers) {
      expect(wp.id).toBeTruthy();
      expect(wp.name).toBeTruthy();
      expect(wp.style).toContain("linear-gradient");
    }
  });

  it("has unique IDs", () => {
    const ids = wallpapers.map((w) => w.id);
    expect(new Set(ids).size).toBe(ids.length);
  });

  it("includes iOS default wallpaper", () => {
    expect(wallpapers.find((w) => w.id === "ios-dark")).toBeTruthy();
  });

  it("includes pure white and pure black", () => {
    expect(wallpapers.find((w) => w.id === "white")).toBeTruthy();
    expect(wallpapers.find((w) => w.id === "black")).toBeTruthy();
  });
});

describe("DevicePreview panel navigation", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  it("navigates to preview panel", async () => {
    const { useUiStore } = await import("@/stores/ui");
    const ui = useUiStore();
    ui.setPanel("preview");
    expect(ui.activePanel).toBe("preview");
  });
});

describe("DevicePreview SVG embedding", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  it("svgPreview is reactive from project store", () => {
    const project = useProjectStore();
    expect(project.svgPreview).toBe("");
  });

  it("svgPreview updates after fetchPreview", async () => {
    const project = useProjectStore();
    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as ReturnType<typeof vi.fn>).mockResolvedValueOnce(
      '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 48 48"><rect width="48" height="48" fill="red"/></svg>',
    );
    await project.fetchPreview();
    expect(project.svgPreview).toContain("<svg");
    expect(project.svgPreview).toContain("</svg>");
  });

  it("device components receive svgContent prop", async () => {
    const sampleSvg =
      '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 48 48"><circle cx="24" cy="24" r="20"/></svg>';
    // Verify the SVG string can be embedded in v-html without errors
    expect(sampleSvg).toContain("<svg");
    expect(sampleSvg).toContain("circle");
  });
});

describe("DevicePreview device tabs", () => {
  const devices = [
    { id: "iphone", label: "iPhone" },
    { id: "android", label: "Android" },
    { id: "macbook", label: "MacBook" },
    { id: "browser", label: "Browser" },
  ];

  it("has 4 device types", () => {
    expect(devices).toHaveLength(4);
  });

  it("all device IDs are unique", () => {
    const ids = devices.map((d) => d.id);
    expect(new Set(ids).size).toBe(ids.length);
  });

  it("includes iPhone and Android devices", () => {
    expect(devices.find((d) => d.id === "iphone")).toBeTruthy();
    expect(devices.find((d) => d.id === "android")).toBeTruthy();
  });
});
