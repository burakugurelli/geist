import { describe, expect, it } from "vitest";

import demoHtml from "../../examples/web/index.html?raw";

describe("minimal web preview", () => {
  it("shows only the three cue buttons as visible page content", () => {
    expect(demoHtml).toContain(
      '<main class="preview" aria-label="Geist sound preview">',
    );
    expect(demoHtml).toContain(
      '<button id="play-press" type="button">Press</button>',
    );
    expect(demoHtml).toContain(
      '<button id="play-success" type="button">Success</button>',
    );
    expect(demoHtml).toContain(
      '<button id="play-error" type="button">Error</button>',
    );
    expect(demoHtml).toContain('id="playback-status"');
    expect(demoHtml).toContain('class="sr-only"');
    expect(demoHtml).toContain('role="status"');
    expect(demoHtml).toContain('aria-live="polite"');
    expect(demoHtml).not.toMatch(/<(?:header|footer|h1|section)\b/);
  });
});
