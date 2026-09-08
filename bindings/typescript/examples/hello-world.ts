import { initialize, cleanup, Renderer, Terminal } from '../src';

initialize();
try {
  const terminal = new Terminal();
  const size = terminal.getSize();
  terminal.dispose();
  const renderer = new Renderer(size.width, size.height);
  try {
    const surface = renderer.getSurface();
    renderer.frame(() => {
      surface.clear(15, 20, 30);
      for (const [x, ch] of Array.from('Hello, terminal!').entries()) {
        if (x < size.width) surface.setCell(x, 0, ch, 0xFFFFFF, 0x0F141E);
      }
    });
    surface.dispose();
  } finally { renderer.dispose(); }
} finally { cleanup(); }
