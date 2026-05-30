import { describe, expect, it } from "vitest";
import { formatTimestamp, initials, relativeTime } from "./format";

describe("format helpers", () => {
  it("derives initials", () => {
    expect(initials("Elena Marchetti")).toBe("EM");
    expect(initials("Daniel")).toBe("D");
    expect(initials("  aoife  brennan ")).toBe("AB");
  });

  it("formats an ISO timestamp to YYYY-MM-DD HH:MM:SS (UTC)", () => {
    expect(formatTimestamp("2026-05-30T09:42:11Z")).toBe("2026-05-30 09:42:11");
  });

  it("returns coarse relative times", () => {
    const now = Date.parse("2026-05-30T12:00:00Z");
    expect(relativeTime(null, now)).toBe("—");
    expect(relativeTime("2026-05-30T11:59:30Z", now)).toBe("Just now");
    expect(relativeTime("2026-05-30T11:58:00Z", now)).toBe("2m ago");
    expect(relativeTime("2026-05-30T09:00:00Z", now)).toBe("3h ago");
    expect(relativeTime("2026-05-28T12:00:00Z", now)).toBe("2d ago");
  });
});
