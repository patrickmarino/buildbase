// Lucide-style icons (1.5px stroke, currentColor), ported from the prototype's
// icons.jsx. `I[name]` is a React component.

import React from "react";

type El = string | { t: keyof React.JSX.IntrinsicElements; p: Record<string, unknown> };

function mk(paths: El[]) {
  return function Icon(props: React.SVGProps<SVGSVGElement>) {
    return (
      <svg
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        strokeWidth={1.5}
        strokeLinecap="round"
        strokeLinejoin="round"
        {...props}
      >
        {paths.map((d, i) =>
          typeof d === "string"
            ? <path key={i} d={d} />
            : React.createElement(d.t, { key: i, ...d.p }),
        )}
      </svg>
    );
  };
}

export const I = {
  users: mk(["M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2", { t: "circle", p: { cx: 9, cy: 7, r: 4 } }, "M22 21v-2a4 4 0 0 0-3-3.87", "M16 3.13a4 4 0 0 1 0 7.75"]),
  shield: mk(["M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z", "M9 12l2 2 4-4"]),
  building: mk([{ t: "rect", p: { x: 4, y: 3, width: 16, height: 18, rx: 1 } }, "M9 9h.01", "M15 9h.01", "M9 13h.01", "M15 13h.01", "M10 21v-3a2 2 0 0 1 4 0v3"]),
  scroll: mk(["M8 21h9a2 2 0 0 0 2-2V6l-3-3H8a2 2 0 0 0-2 2v3", "M14 3v4a1 1 0 0 0 1 1h4", "M9 13h6", "M9 17h4"]),
  key: mk([{ t: "circle", p: { cx: 7.5, cy: 15.5, r: 4.5 } }, "M10.7 12.3 21 2", "M16.5 6.5 19 9"]),
  search: mk([{ t: "circle", p: { cx: 11, cy: 11, r: 7 } }, "M21 21l-4.3-4.3"]),
  filter: mk(["M22 3H2l8 9.46V19l4 2v-8.54L22 3z"]),
  plus: mk(["M12 5v14", "M5 12h14"]),
  check: mk(["M20 6 9 17l-5-5"]),
  minus: mk(["M5 12h14"]),
  x: mk(["M18 6 6 18", "M6 6l12 12"]),
  chevronDown: mk(["M6 9l6 6 6-6"]),
  chevronRight: mk(["M9 6l6 6-6 6"]),
  copy: mk([{ t: "rect", p: { x: 9, y: 9, width: 13, height: 13, rx: 2 } }, "M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"]),
  more: mk([{ t: "circle", p: { cx: 12, cy: 5, r: 1 } }, { t: "circle", p: { cx: 12, cy: 12, r: 1 } }, { t: "circle", p: { cx: 12, cy: 19, r: 1 } }]),
  download: mk(["M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4", "M7 10l5 5 5-5", "M12 15V3"]),
  alert: mk(["M10.29 3.86 1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z", "M12 9v4", "M12 17h.01"]),
  lock: mk([{ t: "rect", p: { x: 3, y: 11, width: 18, height: 11, rx: 2 } }, "M7 11V7a5 5 0 0 1 10 0v4"]),
  mail: mk([{ t: "rect", p: { x: 2, y: 4, width: 20, height: 16, rx: 2 } }, "M22 7l-10 6L2 7"]),
  trash: mk(["M3 6h18", "M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6", "M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"]),
  sliders: mk(["M4 21v-7", "M4 10V3", "M12 21v-9", "M12 8V3", "M20 21v-5", "M20 12V3", "M1 14h6", "M9 8h6", "M17 16h6"]),
  arrowRight: mk(["M5 12h14", "M12 5l7 7-7 7"]),
  eye: mk(["M2 12s3.5-7 10-7 10 7 10 7-3.5 7-10 7-10-7-10-7z", { t: "circle", p: { cx: 12, cy: 12, r: 3 } }]),
  refresh: mk(["M3 12a9 9 0 0 1 15-6.7L21 8", "M21 3v5h-5", "M21 12a9 9 0 0 1-15 6.7L3 16", "M3 21v-5h5"]),
  shieldOff: mk(["M19.7 14a8.5 8.5 0 0 0 .3-2V5l-8-3-3.3 1.2", "M4.7 4.7 4 5v7c0 6 8 10 8 10a14 14 0 0 0 4.7-3", "M3 3l18 18"]),
  edit: mk(["M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7", "M18.5 2.5a2.1 2.1 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"]),
  userX: mk(["M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2", { t: "circle", p: { cx: 9, cy: 7, r: 4 } }, "M17 8l5 5", "M22 8l-5 5"]),
  globe: mk([{ t: "circle", p: { cx: 12, cy: 12, r: 10 } }, "M2 12h20", "M12 2a15 15 0 0 1 0 20 15 15 0 0 1 0-20"]),
  link: mk(["M10 13a5 5 0 0 0 7 0l3-3a5 5 0 0 0-7-7l-1 1", "M14 11a5 5 0 0 0-7 0l-3 3a5 5 0 0 0 7 7l1-1"]),
  fingerprint: mk(["M12 10a2 2 0 0 0-2 2c0 1.5.5 3-1 5", "M2 12a10 10 0 0 1 18-6", "M2 16h.01", "M21.8 16c.2-2 .131-5.354 0-6", "M5 19.5C5.5 18 6 15 6 12a6 6 0 0 1 .34-2", "M8.65 22c.21-.66.45-1.32.57-2", "M14 13.12c0 2.38 0 6.38-1 8.88", "M17.29 21.02c.12-.6.43-2.3.5-3.02"]),
  dot: mk([{ t: "circle", p: { cx: 12, cy: 12, r: 4, fill: "currentColor", stroke: "none" } }]),
} as const;

export type IconName = keyof typeof I;
