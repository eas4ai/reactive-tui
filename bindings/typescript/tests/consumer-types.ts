// A downstream Node project must not need resolveJsonModule or skipLibCheck.
import { Terminal, Surface, ComponentBuilder, lib, getVersion } from '../dist';

export function typedConsumer(terminal: Terminal, surface: Surface): number {
  const component = new ComponentBuilder('div').child(new ComponentBuilder('span').text('hello')).build();
  try {
    surface.setCell(0, 0, 'X');
    lib.rtui_terminal_get_dimensions(terminal.getNativeHandle(), null);
    return getVersion().abiVersion;
  } finally { component.dispose(); }
}
