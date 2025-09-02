/**
 * Component Demo
 * Demonstrates the component system and builder pattern
 */

import {
  initialize,
  cleanup,
  Terminal,
  Renderer,
  Surface,
  Component,
  ComponentBuilder,
  div,
  text,
  button,
  input,
  flex,
  grid,
} from '@reactive-tui/core';

async function createUI(): Promise<Component> {
  // Build a complex UI using the builder pattern
  const ui = div()
    .prop('id', 'main-container')
    .prop('style', {
      padding: 2,
      border: 'rounded',
      backgroundColor: '#282a36',
    })
    .children(
      // Header
      flex()
        .prop('direction', 'horizontal')
        .prop('justify', 'space-between')
        .prop('style', { marginBottom: 1 })
        .children(
          text('🚀 Component Demo'),
          text('v1.0.0')
        ),
      
      // Content grid
      grid()
        .prop('columns', 2)
        .prop('gap', 2)
        .prop('style', { marginBottom: 2 })
        .children(
          // Left panel
          div()
            .prop('style', {
              border: 'solid',
              padding: 1,
              borderColor: '#44475a',
            })
            .children(
              text('User Information').prop('style', { 
                fontWeight: 'bold',
                color: '#8be9fd',
                marginBottom: 1,
              }),
              
              input('Enter your name...').prop('id', 'name-input'),
              input('Enter your email...').prop('id', 'email-input'),
              
              button('Submit').prop('style', {
                marginTop: 1,
                backgroundColor: '#50fa7b',
                color: '#282a36',
              })
            ),
          
          // Right panel
          div()
            .prop('style', {
              border: 'solid',
              padding: 1,
              borderColor: '#44475a',
            })
            .children(
              text('Options').prop('style', {
                fontWeight: 'bold',
                color: '#ffb86c',
                marginBottom: 1,
              }),
              
              button('Option 1').prop('style', { marginBottom: 0.5 }),
              button('Option 2').prop('style', { marginBottom: 0.5 }),
              button('Option 3').prop('style', { marginBottom: 0.5 }),
              button('Clear All').prop('style', {
                backgroundColor: '#ff5555',
                color: '#f8f8f2',
              })
            )
        ),
      
      // Footer
      flex()
        .prop('direction', 'horizontal')
        .prop('justify', 'center')
        .prop('style', {
          borderTop: 'solid',
          paddingTop: 1,
          borderColor: '#44475a',
        })
        .children(
          text('Press ESC to exit | TAB to navigate | ENTER to select')
            .prop('style', { color: '#6272a4' })
        )
    );

  return ui.build();
}

async function main() {
  initialize();

  let terminal: Terminal | null = null;
  let renderer: Renderer | null = null;
  let surface: Surface | null = null;
  let rootComponent: Component | null = null;

  try {
    // Setup terminal
    terminal = new Terminal(120, 40);
    terminal.init();
    
    const size = terminal.getSize();
    renderer = new Renderer(size.width, size.height);
    surface = new Surface(size.width, size.height);

    // Create the UI
    rootComponent = await createUI();

    // Initial render
    renderer.frame(() => {
      surface.clear(40, 42, 54); // Dracula background
      rootComponent!.render(surface);
    });

    // Event handling simulation
    let focusedIndex = 0;
    const focusableElements = ['name-input', 'email-input', 'submit-btn'];

    // Simulate keyboard navigation
    process.stdin.setRawMode(true);
    process.stdin.resume();
    process.stdin.setEncoding('utf8');

    process.stdin.on('data', (key: string) => {
      // Handle key events
      if (key === '\u001b') { // ESC
        console.log('\nExiting...');
        process.exit(0);
      } else if (key === '\t') { // TAB
        focusedIndex = (focusedIndex + 1) % focusableElements.length;
        console.log(`Focus moved to: ${focusableElements[focusedIndex]}`);
        
        // Update component focus (simulation)
        rootComponent!.update({
          focusedElement: focusableElements[focusedIndex]
        });
        
        // Re-render
        renderer!.frame(() => {
          surface!.clear(40, 42, 54);
          rootComponent!.render(surface!);
        });
      } else if (key === '\r') { // ENTER
        console.log(`Activated: ${focusableElements[focusedIndex]}`);
        
        // Handle activation
        rootComponent!.handleEvent({
          type: 'click',
          target: focusableElements[focusedIndex],
        });
      }
    });

    console.log('Component demo running.');
    console.log('Controls:');
    console.log('  TAB    - Navigate between fields');
    console.log('  ENTER  - Activate focused element');
    console.log('  ESC    - Exit application');

    // Keep process alive
    await new Promise(() => {});

  } catch (error) {
    console.error('Error:', error);
  } finally {
    // Cleanup
    if (rootComponent) rootComponent.dispose();
    if (surface) surface.dispose();
    if (renderer) renderer.dispose();
    if (terminal) terminal.dispose();
    cleanup();
    
    // Restore terminal
    if (process.stdin.setRawMode) {
      process.stdin.setRawMode(false);
    }
  }
}

main().catch(console.error);