import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { mount } from "@vue/test-utils";
import { createPinia, setActivePinia } from "pinia";
import { defineComponent, h } from "vue";
import { useUiStore } from "@/stores/ui";
import { useProjectStore } from "@/stores/project";
import { useElementsStore } from "@/stores/elements";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue(undefined),
}));
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
}));
vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: vi.fn().mockResolvedValue(null),
}));

import { useKeyboard } from "@/composables/useKeyboard";

function makeElement(id: string, overrides: Record<string, unknown> = {}) {
  return {
    id,
    type: "shape" as const,
    shape_type: "rect" as const,
    x: 100,
    y: 100,
    width: 50,
    height: 50,
    fill: "#FF0000",
    stroke: null,
    stroke_width: 0,
    opacity: 1,
    rotation: 0,
    border_radius: 0,
    gradient: null,
    shadows: [],
    animation: null,
    locked: false,
    visible: true,
    ...overrides,
  };
}

let dispatchTarget: HTMLElement;

function setupDispatchTarget() {
  dispatchTarget = document.createElement("div");
  document.body.appendChild(dispatchTarget);
}

function cleanupDispatchTarget() {
  dispatchTarget?.remove();
}

function pressKey(key: string, options: KeyboardEventInit = {}) {
  const event = new KeyboardEvent("keydown", { key, bubbles: true, ...options });
  vi.spyOn(event, "preventDefault").mockImplementation(() => {});
  dispatchTarget.dispatchEvent(event);
  return event;
}

describe("useKeyboard", () => {
  let wrapper: ReturnType<typeof mount>;
  let ui: ReturnType<typeof useUiStore>;
  let project: ReturnType<typeof useProjectStore>;
  let elementsStore: ReturnType<typeof useElementsStore>;

  beforeEach(() => {
    setupDispatchTarget();
    const pinia = createPinia();
    setActivePinia(pinia);
    wrapper = mount(
      defineComponent({
        setup() {
          useKeyboard();
          return () => h("div");
        },
      }),
      { global: { plugins: [pinia] } }
    );
    ui = useUiStore();
    project = useProjectStore();
    elementsStore = useElementsStore();
  });

  afterEach(() => {
    wrapper.unmount();
    cleanupDispatchTarget();
  });

  it("deletes selected element on Delete key", () => {
    const el = makeElement("el-1");
    elementsStore.items = [el] as any;
    ui.selectElement("el-1");

    const spy = vi.spyOn(project, "removeElement");
    pressKey("Delete");

    expect(spy).toHaveBeenCalledWith("el-1");
    expect(ui.selectedElementId).toBeNull();
  });

  it("deletes selected element on Backspace key", () => {
    const el = makeElement("el-1");
    elementsStore.items = [el] as any;
    ui.selectElement("el-1");

    const spy = vi.spyOn(project, "removeElement");
    pressKey("Backspace");

    expect(spy).toHaveBeenCalledWith("el-1");
    expect(ui.selectedElementId).toBeNull();
  });

  it("does NOT delete locked element", () => {
    const el = makeElement("el-1", { locked: true });
    elementsStore.items = [el] as any;
    ui.selectElement("el-1");

    const spy = vi.spyOn(project, "removeElement");
    pressKey("Delete");

    expect(spy).not.toHaveBeenCalled();
  });

  it("duplicates selected element on Ctrl+D", () => {
    const el = makeElement("el-1");
    elementsStore.items = [el] as any;
    ui.selectElement("el-1");

    const spy = vi.spyOn(project, "duplicateElement");
    pressKey("d", { ctrlKey: true });

    expect(spy).toHaveBeenCalledWith("el-1");
  });

  it("nudges element by 1px on ArrowRight", () => {
    const el = makeElement("el-1", { x: 100, y: 100 });
    elementsStore.items = [el] as any;
    ui.selectElement("el-1");

    const spy = vi.spyOn(project, "updateElement");
    pressKey("ArrowRight");

    expect(spy).toHaveBeenCalledWith("el-1", { x: 101, y: 100 });
  });

  it("nudges element by 10px with Shift+ArrowDown", () => {
    const el = makeElement("el-1", { x: 100, y: 100 });
    elementsStore.items = [el] as any;
    ui.selectElement("el-1");

    const spy = vi.spyOn(project, "updateElement");
    pressKey("ArrowDown", { shiftKey: true });

    expect(spy).toHaveBeenCalledWith("el-1", { x: 100, y: 110 });
  });

  it("does NOT move locked element with arrow keys", () => {
    const el = makeElement("el-1", { x: 100, y: 100, locked: true });
    elementsStore.items = [el] as any;
    ui.selectElement("el-1");

    const spy = vi.spyOn(project, "updateElement");
    pressKey("ArrowRight");

    expect(spy).not.toHaveBeenCalled();
  });

  it("suppresses shortcuts when input is focused", () => {
    const input = document.createElement("input");
    dispatchTarget.appendChild(input);

    const el = makeElement("el-1");
    elementsStore.items = [el] as any;
    ui.selectElement("el-1");

    const spy = vi.spyOn(project, "removeElement");
    const event = new KeyboardEvent("keydown", { key: "Delete", bubbles: true });
    vi.spyOn(event, "preventDefault").mockImplementation(() => {});
    input.dispatchEvent(event);

    expect(spy).not.toHaveBeenCalled();
  });
});
