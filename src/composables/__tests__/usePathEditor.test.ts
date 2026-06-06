import { describe, it, expect } from "vitest";
import { parsePath, serializePath } from "../usePathEditor";

describe("parsePath", () => {
  it("parses a simple line (M + L)", () => {
    const nodes = parsePath("M 10 10 L 100 100");
    expect(nodes.length).toBe(2);
    expect(nodes[0].command).toBe("M");
    expect(nodes[0].anchor).toEqual({ x: 10, y: 10 });
    expect(nodes[1].command).toBe("L");
    expect(nodes[1].anchor).toEqual({ x: 100, y: 100 });
  });

  it("parses a cubic bezier curve", () => {
    const nodes = parsePath("M 0 0 C 30 30 70 30 100 0");
    expect(nodes.length).toBe(2);
    expect(nodes[0].command).toBe("M");
    expect(nodes[0].anchor).toEqual({ x: 0, y: 0 });
    expect(nodes[1].command).toBe("C");
    expect(nodes[1].anchor).toEqual({ x: 100, y: 0 });
    expect(nodes[1].handleIn).toEqual({ x: 30, y: 30 });
    expect(nodes[1].handleOut).toEqual({ x: 70, y: 30 });
  });

  it("parses a closed path with Z", () => {
    const nodes = parsePath("M 0 0 L 100 0 L 100 100 Z");
    expect(nodes.length).toBe(4);
    expect(nodes[0]).toEqual({ command: "M", anchor: { x: 0, y: 0 } });
    expect(nodes[1]).toEqual({ command: "L", anchor: { x: 100, y: 0 } });
    expect(nodes[2]).toEqual({ command: "L", anchor: { x: 100, y: 100 } });
    expect(nodes[3].command).toBe("Z");
  });

  it("parses implicit lineTo after moveTo (numbers after M without L)", () => {
    const nodes = parsePath("M 10 10 100 100");
    expect(nodes.length).toBe(2);
    expect(nodes[0].command).toBe("M");
    expect(nodes[1].command).toBe("L");
    expect(nodes[1].anchor).toEqual({ x: 100, y: 100 });
  });

  it("handles relative commands", () => {
    const nodes = parsePath("M 10 10 l 20 20");
    expect(nodes.length).toBe(2);
    expect(nodes[0].anchor).toEqual({ x: 10, y: 10 });
    expect(nodes[1].command).toBe("L");
    expect(nodes[1].anchor).toEqual({ x: 30, y: 30 });
  });

  it("handles H and V commands", () => {
    const nodes = parsePath("M 10 20 H 100 V 50");
    expect(nodes.length).toBe(3);
    expect(nodes[0]).toEqual({ command: "M", anchor: { x: 10, y: 20 } });
    expect(nodes[1]).toEqual({ command: "L", anchor: { x: 100, y: 20 } });
    expect(nodes[2]).toEqual({ command: "L", anchor: { x: 100, y: 50 } });
  });

  it("converts arc (A) command to cubic bezier", () => {
    const nodes = parsePath("M 10 80 A 45 45 0 0 0 125 125");
    expect(nodes.length).toBeGreaterThanOrEqual(2);
    expect(nodes[0].command).toBe("M");
    for (let i = 1; i < nodes.length; i++) {
      expect(nodes[i].command).toBe("C");
      expect(nodes[i].handleIn).toBeDefined();
      expect(nodes[i].handleOut).toBeDefined();
    }
    const last = nodes[nodes.length - 1];
    expect(last.anchor.x).toBeCloseTo(125, 1);
    expect(last.anchor.y).toBeCloseTo(125, 1);
  });

  it("converts degenerate arc (zero radius) to line", () => {
    const nodes = parsePath("M 0 0 A 0 0 0 0 1 50 50");
    expect(nodes.length).toBe(2);
    expect(nodes[0].command).toBe("M");
    expect(nodes[1].command).toBe("L");
    expect(nodes[1].anchor).toEqual({ x: 50, y: 50 });
  });

  it("handles Q (quadratic bezier) by converting to cubic", () => {
    const nodes = parsePath("M 0 0 Q 50 0 100 50");
    expect(nodes.length).toBe(2);
    expect(nodes[1].command).toBe("C");
    expect(nodes[1].handleIn).toBeDefined();
    expect(nodes[1].handleOut).toBeDefined();
    expect(nodes[1].anchor.x).toBeCloseTo(100, 1);
    expect(nodes[1].anchor.y).toBeCloseTo(50, 1);
  });

  it("returns empty array for empty string", () => {
    expect(parsePath("")).toEqual([]);
    expect(parsePath("   ")).toEqual([]);
  });

  it("returns empty array for input with only non-path letters", () => {
    expect(parsePath("hello world")).toEqual([]);
  });

  it("handles comma-separated numbers", () => {
    const nodes = parsePath("M10,20L30,40");
    expect(nodes.length).toBe(2);
    expect(nodes[0].anchor).toEqual({ x: 10, y: 20 });
    expect(nodes[1].anchor).toEqual({ x: 30, y: 40 });
  });
});

describe("serializePath", () => {
  it("serializes M and L nodes", () => {
    const d = serializePath([
      { command: "M", anchor: { x: 10, y: 10 } },
      { command: "L", anchor: { x: 100, y: 100 } },
    ]);
    expect(d).toBe("M10,10 L100,100");
  });

  it("serializes cubic bezier C nodes", () => {
    const d = serializePath([
      { command: "M", anchor: { x: 0, y: 0 } },
      {
        command: "C",
        anchor: { x: 100, y: 0 },
        handleIn: { x: 30, y: 30 },
        handleOut: { x: 70, y: 30 },
      },
    ]);
    expect(d).toContain("C30,30 70,30 100,0");
  });

  it("serializes Z (close path)", () => {
    const d = serializePath([
      { command: "M", anchor: { x: 0, y: 0 } },
      { command: "L", anchor: { x: 100, y: 0 } },
      { command: "Z", anchor: { x: 0, y: 0 } },
    ]);
    expect(d).toContain("Z");
    expect(d).toBe("M0,0 L100,0 Z");
  });

  it("returns empty string for empty array", () => {
    expect(serializePath([])).toBe("");
  });
});

describe("parse/serialize roundtrip", () => {
  it("roundtrips a simple line path", () => {
    const original = "M10,10 L100,100";
    const nodes = parsePath(original);
    const serialized = serializePath(nodes);
    const reparsed = parsePath(serialized);
    expect(reparsed.length).toBe(nodes.length);
    for (let i = 0; i < nodes.length; i++) {
      expect(reparsed[i].command).toBe(nodes[i].command);
      expect(reparsed[i].anchor.x).toBeCloseTo(nodes[i].anchor.x, 1);
      expect(reparsed[i].anchor.y).toBeCloseTo(nodes[i].anchor.y, 1);
    }
  });

  it("roundtrips a cubic bezier path", () => {
    const original = "M0,0 C30,30 70,30 100,0";
    const nodes = parsePath(original);
    const serialized = serializePath(nodes);
    const reparsed = parsePath(serialized);
    expect(reparsed.length).toBe(nodes.length);
    expect(reparsed[1].command).toBe("C");
    expect(reparsed[1].handleIn!.x).toBeCloseTo(30, 1);
    expect(reparsed[1].handleOut!.y).toBeCloseTo(30, 1);
    expect(reparsed[1].anchor.x).toBeCloseTo(100, 1);
  });

  it("roundtrips a closed triangle", () => {
    const original = "M0,0 L100,0 L100,100 Z";
    const nodes = parsePath(original);
    const serialized = serializePath(nodes);
    expect(serialized).toContain("Z");
    const reparsed = parsePath(serialized);
    expect(reparsed.length).toBe(4);
    expect(reparsed[3].command).toBe("Z");
  });
});
