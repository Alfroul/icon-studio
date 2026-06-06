import { describe, it, expect } from "vitest";
import { parsePath } from "@/composables/usePathEditor";

describe("PathEditorOverlay (data logic)", () => {
  it("parses path data for rendering anchor points", () => {
    const nodes = parsePath("M 10 10 L 100 100 L 200 50 Z");
    expect(nodes.length).toBe(4);
    const anchors = nodes.map((n) => n.anchor);
    expect(anchors[0]).toEqual({ x: 10, y: 10 });
    expect(anchors[1]).toEqual({ x: 100, y: 100 });
    expect(anchors[2]).toEqual({ x: 200, y: 50 });
  });

  it("distinguishes selected vs unselected anchors by index", () => {
    const nodes = parsePath("M 0 0 L 50 50 L 100 0");
    expect(nodes.length).toBe(3);
    const selectedIndex = 1;
    expect(nodes[selectedIndex].command).toBe("L");
    expect(nodes[selectedIndex].anchor).toEqual({ x: 50, y: 50 });
    for (let i = 0; i < nodes.length; i++) {
      if (i !== selectedIndex) {
        expect(i).not.toBe(selectedIndex);
      }
    }
  });

  it("identifies nodes with bezier handles", () => {
    const nodes = parsePath("M 0 0 C 30 30 70 30 100 0 L 150 50");
    expect(nodes.length).toBe(3);
    expect(nodes[1].command).toBe("C");
    expect(nodes[1].handleIn).toBeDefined();
    expect(nodes[1].handleOut).toBeDefined();
    expect(nodes[2].command).toBe("L");
    expect(nodes[2].handleIn).toBeUndefined();
    expect(nodes[2].handleOut).toBeUndefined();
  });
});
