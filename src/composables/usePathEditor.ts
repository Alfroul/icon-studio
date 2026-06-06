import type { PathNode } from "@/types";

/**
 * Parse an SVG path `d` attribute string into an array of PathNode objects.
 * Supports M, L, C, Q, Z commands. Arc (A) commands are converted to cubic bezier approximations.
 */
export function parsePath(d: string): PathNode[] {
  if (!d || typeof d !== "string") return [];

  // Tokenize: extract command letters and numbers
  const tokens: { type: "command"; value: string } | { type: "number"; value: number }[] = [];
  const len = d.length;
  let i = 0;

  while (i < len) {
    const ch = d[i];

    // Skip whitespace and commas
    if (ch === " " || ch === "\t" || ch === "\n" || ch === "\r" || ch === ",") {
      i++;
      continue;
    }

    // Command letter
    if (/[a-zA-Z]/.test(ch)) {
      tokens.push({ type: "command", value: ch });
      i++;
      continue;
    }

    // Number: optional sign, digits, optional decimal, optional exponent
    const start = i;
    if (d[i] === "-" || d[i] === "+") i++;
    let hasDot = false;
    while (i < len) {
      if (d[i] >= "0" && d[i] <= "9") {
        i++;
      } else if (d[i] === "." && !hasDot) {
        hasDot = true;
        i++;
      } else {
        break;
      }
    }
    // Handle exponent: e/E followed by optional sign and digits
    if (i < len && (d[i] === "e" || d[i] === "E")) {
      i++;
      if (i < len && (d[i] === "-" || d[i] === "+")) i++;
      while (i < len && d[i] >= "0" && d[i] <= "9") i++;
    }
    const numStr = d.slice(start, i);
    if (numStr === "" || numStr === "-" || numStr === "+" || numStr === ".") {
      i = start + 1;
      continue;
    }
    const num = parseFloat(numStr);
    if (!isNaN(num)) {
      tokens.push({ type: "number", value: num });
    }
  }

  if (tokens.length === 0) return [];

  const nodes: PathNode[] = [];
  let cx = 0; // current x
  let cy = 0; // current y
  let sx = 0; // subpath start x (for Z)
  let sy = 0; // subpath start y

  let t = 0;
  let currentCmd = "";

  function consumeNumber(): number | null {
    while (t < tokens.length && tokens[t].type !== "number") t++;
    if (t >= tokens.length) return null;
    return (tokens[t++] as { type: "number"; value: number }).value;
  }

  while (t < tokens.length) {
    const tok = tokens[t];

    // If it's a command letter, update currentCmd
    if (tok.type === "command") {
      currentCmd = tok.value;
      t++;
    }

    const isRelative = currentCmd === currentCmd.toLowerCase();
    const cmdUpper = currentCmd.toUpperCase();

    switch (cmdUpper) {
      case "M": {
        const x = consumeNumber();
        const y = consumeNumber();
        if (x === null || y === null) continue;
        const ax = isRelative ? cx + x : x;
        const ay = isRelative ? cy + y : y;
        nodes.push({ command: "M", anchor: { x: ax, y: ay } });
        cx = ax;
        cy = ay;
        sx = ax;
        sy = ay;
        // Subsequent coordinate pairs after M are treated as L
        currentCmd = isRelative ? "l" : "L";
        break;
      }
      case "L": {
        const x = consumeNumber();
        const y = consumeNumber();
        if (x === null || y === null) continue;
        const ax = isRelative ? cx + x : x;
        const ay = isRelative ? cy + y : y;
        nodes.push({ command: "L", anchor: { x: ax, y: ay } });
        cx = ax;
        cy = ay;
        break;
      }
      case "C": {
        const x1 = consumeNumber();
        const y1 = consumeNumber();
        const x2 = consumeNumber();
        const y2 = consumeNumber();
        const x = consumeNumber();
        const y = consumeNumber();
        if (x1 === null || y1 === null || x2 === null || y2 === null || x === null || y === null) continue;
        const cp1x = isRelative ? cx + x1 : x1;
        const cp1y = isRelative ? cy + y1 : y1;
        const cp2x = isRelative ? cx + x2 : x2;
        const cp2y = isRelative ? cy + y2 : y2;
        const ax = isRelative ? cx + x : x;
        const ay = isRelative ? cy + y : y;
        nodes.push({
          command: "C",
          anchor: { x: ax, y: ay },
          handleIn: { x: cp1x, y: cp1y },
          handleOut: { x: cp2x, y: cp2y },
        });
        cx = ax;
        cy = ay;
        break;
      }
      case "Q": {
        const qx = consumeNumber();
        const qy = consumeNumber();
        const x = consumeNumber();
        const y = consumeNumber();
        if (qx === null || qy === null || x === null || y === null) continue;
        // Convert quadratic to cubic
        const qcpX = isRelative ? cx + qx : qx;
        const qcpY = isRelative ? cy + qy : qy;
        const ax = isRelative ? cx + x : x;
        const ay = isRelative ? cy + y : y;
        // Quadratic → Cubic: cp1 = start + 2/3*(qcp-start), cp2 = end + 2/3*(qcp-end)
        const cp1x = cx + (2 / 3) * (qcpX - cx);
        const cp1y = cy + (2 / 3) * (qcpY - cy);
        const cp2x = ax + (2 / 3) * (qcpX - ax);
        const cp2y = ay + (2 / 3) * (qcpY - ay);
        nodes.push({
          command: "C",
          anchor: { x: ax, y: ay },
          handleIn: { x: cp1x, y: cp1y },
          handleOut: { x: cp2x, y: cp2y },
        });
        cx = ax;
        cy = ay;
        break;
      }
      case "Z": {
        nodes.push({ command: "Z", anchor: { x: sx, y: sy } });
        cx = sx;
        cy = sy;
        // Consume no parameters; if next token is a command, loop will pick it up
        // If another Z follows with implicit repeat, just loop again
        break;
      }
      case "A": {
        const rx = consumeNumber();
        const ry = consumeNumber();
        const angle = consumeNumber();
        const largeArcFlag = consumeNumber();
        const sweepFlag = consumeNumber();
        const x = consumeNumber();
        const y = consumeNumber();
        if (rx === null || ry === null || angle === null || largeArcFlag === null || sweepFlag === null || x === null || y === null) continue;
        const ax = isRelative ? cx + x : x;
        const ay = isRelative ? cy + y : y;
        const arcNodes = arcToBezier(
          cx, cy,
          Math.abs(rx), Math.abs(ry),
          angle,
          largeArcFlag !== 0,
          sweepFlag !== 0,
          ax, ay
        );
        for (const node of arcNodes) {
          nodes.push(node);
        }
        cx = ax;
        cy = ay;
        break;
      }
      case "H": {
        const x = consumeNumber();
        if (x === null) continue;
        const ax = isRelative ? cx + x : x;
        nodes.push({ command: "L", anchor: { x: ax, y: cy } });
        cx = ax;
        break;
      }
      case "V": {
        const y = consumeNumber();
        if (y === null) continue;
        const ay = isRelative ? cy + y : y;
        nodes.push({ command: "L", anchor: { x: cx, y: ay } });
        cy = ay;
        break;
      }
      case "S": {
        // Smooth cubic: reflect previous handleOut
        const x2 = consumeNumber();
        const y2 = consumeNumber();
        const x = consumeNumber();
        const y = consumeNumber();
        if (x2 === null || y2 === null || x === null || y === null) continue;
        // Derive cp1 by reflecting last cp2
        const lastNode = nodes[nodes.length - 1];
        let cp1x = cx;
        let cp1y = cy;
        if (lastNode && lastNode.command === "C" && lastNode.handleOut) {
          cp1x = 2 * cx - lastNode.handleOut.x;
          cp1y = 2 * cy - lastNode.handleOut.y;
        }
        const cp2x = isRelative ? cx + x2 : x2;
        const cp2y = isRelative ? cy + y2 : y2;
        const ax = isRelative ? cx + x : x;
        const ay = isRelative ? cy + y : y;
        nodes.push({
          command: "C",
          anchor: { x: ax, y: ay },
          handleIn: { x: cp1x, y: cp1y },
          handleOut: { x: cp2x, y: cp2y },
        });
        cx = ax;
        cy = ay;
        break;
      }
      case "T": {
        // Smooth quadratic: reflect previous quadratic control point
        const x = consumeNumber();
        const y = consumeNumber();
        if (x === null || y === null) continue;
        // For simplicity, we treat this as a quadratic with reflected control point then convert to cubic
        // Find last Q-derived control point
        const lastNode = nodes[nodes.length - 1];
        let qcpX = cx;
        let qcpY = cy;
        if (lastNode && lastNode.command === "C" && lastNode.handleIn && lastNode.handleOut) {
          // Approximation: reflect the handleIn as the quadratic control point
          qcpX = 2 * cx - lastNode.handleIn.x;
          qcpY = 2 * cy - lastNode.handleIn.y;
        }
        const ax = isRelative ? cx + x : x;
        const ay = isRelative ? cy + y : y;
        const cp1x = cx + (2 / 3) * (qcpX - cx);
        const cp1y = cy + (2 / 3) * (qcpY - cy);
        const cp2x = ax + (2 / 3) * (qcpX - ax);
        const cp2y = ay + (2 / 3) * (qcpY - ay);
        nodes.push({
          command: "C",
          anchor: { x: ax, y: ay },
          handleIn: { x: cp1x, y: cp1y },
          handleOut: { x: cp2x, y: cp2y },
        });
        cx = ax;
        cy = ay;
        break;
      }
      default:
        // Unknown command, skip
        t++;
        break;
    }
  }

  return nodes;
}

/**
 * Serialize an array of PathNode objects back to an SVG path `d` attribute string.
 */
export function serializePath(nodes: PathNode[]): string {
  if (!nodes || nodes.length === 0) return "";

  const parts: string[] = [];

  for (const node of nodes) {
    switch (node.command) {
      case "M":
        parts.push(`M${r(node.anchor.x)},${r(node.anchor.y)}`);
        break;
      case "L":
        parts.push(`L${r(node.anchor.x)},${r(node.anchor.y)}`);
        break;
      case "C":
        if (node.handleIn && node.handleOut) {
          parts.push(
            `C${r(node.handleIn.x)},${r(node.handleIn.y)} ${r(node.handleOut.x)},${r(node.handleOut.y)} ${r(node.anchor.x)},${r(node.anchor.y)}`
          );
        }
        break;
      case "Q":
        if (node.handleIn) {
          parts.push(`Q${r(node.handleIn.x)},${r(node.handleIn.y)} ${r(node.anchor.x)},${r(node.anchor.y)}`);
        }
        break;
      case "Z":
        parts.push("Z");
        break;
    }
  }

  return parts.join(" ");
}

/** Round to 2 decimal places */
function r(n: number): string {
  return Math.round(n * 100) / 100 + "";
}

/**
 * Convert an SVG arc (A command) to one or more cubic bezier curve approximations.
 * Uses the standard algorithm from SVG spec appendix F.
 */
function arcToBezier(
  x1: number, y1: number,
  rx: number, ry: number,
  angleDeg: number,
  largeArc: boolean,
  sweep: boolean,
  x2: number, y2: number
): PathNode[] {
  // Degenerate cases
  if (rx === 0 || ry === 0) {
    return [{ command: "L", anchor: { x: x2, y: y2 } }];
  }
  // Coincident endpoints
  const dx = x2 - x1;
  const dy = y2 - y1;
  if (dx === 0 && dy === 0) return [];

  const phi = (angleDeg * Math.PI) / 180;
  const cosPhi = Math.cos(phi);
  const sinPhi = Math.sin(phi);

  // Step 1: Compute (x1', y1') — F.6.5.1
  const x1p = (cosPhi * (x1 - x2)) / 2 + (sinPhi * (y1 - y2)) / 2;
  const y1p = (-sinPhi * (x1 - x2)) / 2 + (cosPhi * (y1 - y2)) / 2;

  // Ensure radii are large enough — F.6.6.2
  const lambda = (x1p * x1p) / (rx * rx) + (y1p * y1p) / (ry * ry);
  let rxAdj = rx;
  let ryAdj = ry;
  if (lambda > 1) {
    const sqrtLambda = Math.sqrt(lambda);
    rxAdj *= sqrtLambda;
    ryAdj *= sqrtLambda;
  }

  const rx2 = rxAdj * rxAdj;
  const ry2 = ryAdj * ryAdj;
  const x1p2 = x1p * x1p;
  const y1p2 = y1p * y1p;

  // Step 2: Compute (cx', cy') — F.6.5.2
  let num = rx2 * ry2 - rx2 * y1p2 - ry2 * x1p2;
  let den = rx2 * y1p2 + ry2 * x1p2;
  // Clamp to avoid negative sqrt from floating-point errors
  let sq = den !== 0 ? num / den : 0;
  if (sq < 0) sq = 0;
  const sqrtSq = Math.sqrt(sq);
  const sign = largeArc === sweep ? -1 : 1;
  const cxp = (sign * sqrtSq * rxAdj * y1p) / ryAdj;
  const cyp = (sign * sqrtSq * ryAdj * x1p) / rxAdj;

  // Step 3: Compute (cx, cy) from (cx', cy') — F.6.5.3
  const cx = cosPhi * cxp - sinPhi * cyp + (x1 + x2) / 2;
  const cy = sinPhi * cxp + cosPhi * cyp + (y1 + y2) / 2;

  // Step 4: Compute theta1 and dtheta — F.6.5.5, F.6.5.6
  function vecAngle(ux: number, uy: number, vx: number, vy: number): number {
    const dot = ux * vx + uy * vy;
    const lenU = Math.sqrt(ux * ux + uy * uy);
    const lenV = Math.sqrt(vx * vx + vy * vy);
    let cosA = dot / (lenU * lenV);
    // Clamp
    if (cosA > 1) cosA = 1;
    if (cosA < -1) cosA = -1;
    let a = Math.acos(cosA);
    if (ux * vy - uy * vx < 0) a = -a;
    return a;
  }

  const theta1 = vecAngle(1, 0, (x1p - cxp) / rxAdj, (y1p - cyp) / ryAdj);
  let dTheta = vecAngle(
    (x1p - cxp) / rxAdj, (y1p - cyp) / ryAdj,
    (-x1p - cxp) / rxAdj, (-y1p - cyp) / ryAdj
  );

  // Adjust dTheta per SVG spec
  if (!sweep && dTheta > 0) {
    dTheta -= 2 * Math.PI;
  } else if (sweep && dTheta < 0) {
    dTheta += 2 * Math.PI;
  }

  // Split arc into segments (each ≤ π/2 for good approximation)
  const segments = Math.max(1, Math.ceil(Math.abs(dTheta) / (Math.PI / 2)));
  const dThetaPerSegment = dTheta / segments;

  const nodes: PathNode[] = [];
  let tStart = theta1;

  for (let i = 0; i < segments; i++) {
    const tEnd = tStart + dThetaPerSegment;
    const alpha = (4 / 3) * Math.tan(dThetaPerSegment / 4);

    const cos1 = Math.cos(tStart);
    const sin1 = Math.sin(tStart);
    const cos2 = Math.cos(tEnd);
    const sin2 = Math.sin(tEnd);

    const eLocalX = rxAdj * cos2;
    const eLocalY = ryAdj * sin2;
    const epx = cx + cosPhi * eLocalX - sinPhi * eLocalY;
    const epy = cy + sinPhi * eLocalX + cosPhi * eLocalY;

    // Control point 1 (from start tangent)
    const sLocalX = rxAdj * cos1;
    const sLocalY = ryAdj * sin1;
    const dLocalX1 = -rxAdj * sin1;
    const dLocalY1 = ryAdj * cos1;
    const cp1x = cx + cosPhi * (sLocalX + alpha * dLocalX1) - sinPhi * (sLocalY + alpha * dLocalY1);
    const cp1y = cy + sinPhi * (sLocalX + alpha * dLocalX1) + cosPhi * (sLocalY + alpha * dLocalY1);

    // Control point 2 (from end tangent)
    const dLocalX2 = -rxAdj * sin2;
    const dLocalY2 = ryAdj * cos2;
    const cp2x = cx + cosPhi * (eLocalX - alpha * dLocalX2) - sinPhi * (eLocalY - alpha * dLocalY2);
    const cp2y = cy + sinPhi * (eLocalX - alpha * dLocalX2) + cosPhi * (eLocalY - alpha * dLocalY2);

    nodes.push({
      command: "C",
      anchor: { x: epx, y: epy },
      handleIn: { x: cp1x, y: cp1y },
      handleOut: { x: cp2x, y: cp2y },
    });

    tStart = tEnd;
  }

  return nodes;
}

/**
 * Create a composable for throttled path editing updates.
 * Uses requestAnimationFrame to throttle redraws during drag.
 */
export function usePathEditorThrottle() {
  let rafId: number | null = null;

  function throttleUpdate(callback: () => void) {
    if (rafId !== null) return; // already scheduled
    rafId = requestAnimationFrame(() => {
      callback();
      rafId = null;
    });
  }

  function cancelUpdate() {
    if (rafId !== null) {
      cancelAnimationFrame(rafId);
      rafId = null;
    }
  }

  return { throttleUpdate, cancelUpdate };
}
