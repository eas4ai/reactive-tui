import { initialize, cleanup, Renderer } from '@reactive-tui/core';

initialize();
try {
  const renderer = new Renderer(40, 10);
  try {
    const surface = renderer.getSurface();
    renderer.frame(() => {
      surface.clear(15, 20, 30);
      surface.setCell(2, 2, '界', 0xFFFFFF, 0x0F141E);
    });
    surface.dispose(); // Releases this view, not the renderer's storage.
  } finally { renderer.dispose(); }
} finally { cleanup(); }

export {};
