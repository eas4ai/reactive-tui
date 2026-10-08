import assert from 'node:assert/strict';
import {
  NativeTextEditor, NativeLayoutStyle, NativeDialogEngine, ForeignComponent,
  NativeApp, Component, div, initialize, cleanup,
} from '../../bindings/typescript/src/index';

function units(): void {
  const editor = new NativeTextEditor();
  editor.insert('A界é👩🏽‍💻');
  editor.delete(true);
  assert.equal(editor.content, 'A界é');
  editor.move('left', true);
  editor.insert('!');
  assert.equal(editor.content, 'A界!');
  editor.setSize(12, 2);
  const painted = editor.element();
  editor.dispose(); editor.dispose();
  assert.throws(() => editor.insert('x'));
  painted.dispose();
  assert.throws(() => new NativeLayoutStyle('width:nan'));
  const style = new NativeLayoutStyle('display:flex;gap:3');
  const owned = div().build();
  style.apply(owned); style.dispose(); owned.dispose();
  const dialogs = new NativeDialogEngine();
  for (const kind of ['confirmation', 'input', 'toast', 'progress', 'autocomplete', 'wizard'] as const) {
    const id = dialogs.open({ kind, title: 'Test' });
    assert.equal(dialogs.takeEvent()?.kind, 'opened');
    if (kind === 'progress') dialogs.update(id, { progress: 0.5 });
    if (kind === 'input') dialogs.update(id, { input: 'edited' });
    dialogs.close(id, { kind: 'confirmed', data: 'saved' });
    const result = dialogs.takeEvent();
    assert.equal(result?.kind, 'closed');
    assert.equal(result?.result?.data, 'saved');
    assert.throws(() => dialogs.close(id, { kind: 'cancelled' }));
  }
  assert.equal(dialogs.takeEvent(), null);
  dialogs.dispose(); dialogs.dispose();

  let disposed = 0;
  let component: ForeignComponent<string, number>;
  component = new ForeignComponent({
    props: 'alpha', state: 0,
    render: (props, state) => {
      assert.equal(component.state, state);
      assert.throws(() => component.render());
      assert.throws(() => component.dispose());
      return Component.text(`${props}:${state}`);
    },
    event: () => { component.state += 1; return true; },
    dispose: () => { disposed++; },
  });
  const snapshot = component.element();
  const result = component.render();
  assert.equal(result.getText(), 'alpha:0'); result.dispose();
  assert.equal(component.dispatch({ type: 'custom', name: 'test', data: null }), true);
  assert.equal(component.state, 1);
  assert.throws(() => { component.state = Infinity; });
  assert.equal(component.state, 1);
  component.props = 'omega';
  const next = component.render();
  assert.equal(next.getText(), 'omega:1'); next.dispose();
  component.dispose(); component.dispose(); snapshot.dispose();
  assert.equal(disposed, 1);
  assert.throws(() => component.element());
  const failure = new Error('intentional foreign render failure');
  const broken = new ForeignComponent({ props: null, state: null, render: () => { throw failure; } });
  assert.throws(() => broken.render(), error => error === failure);
  broken.dispose();
  let cleanupCalls = 0;
  const badCleanup = new ForeignComponent({ props: null, state: null,
    render: () => Component.text('unused'),
    dispose: () => { cleanupCalls++; throw new Error('intentional cleanup failure'); },
  });
  assert.throws(() => badCleanup.dispose(), /intentional cleanup failure/);
  badCleanup.dispose();
  assert.equal(cleanupCalls, 1);
  console.log('NATIVE_COMPONENT_UNITS_PASS');
}

function host(mode: string): void {
  const error = mode === 'error';
  const eventError = mode === 'event-error';
  const editor = new NativeTextEditor();
  editor.setSize(24, 2); editor.showLineNumbers(false); editor.insert('Edit:');
  const dialogs = new NativeDialogEngine();
  const rowStyle = new NativeLayoutStyle('display:flex;flex-direction:row;gap:3;height:1;flex-shrink:0');
  if (mode === 'dialog') {
    const options = JSON.parse(process.argv[3]);
    const id = dialogs.open(options);
    if (options.kind === 'progress') dialogs.update(id, { progress: 0.75 });
  }
  let result = '';
  let completion = 'none';
  let disposed = 0;
  let component: ForeignComponent<string, number>;
  component = new ForeignComponent({
    props: 'alpha', state: 0,
    render: (props, state) => {
      if (error) throw new Error('intentional host callback failure');
      let next;
      while ((next = dialogs.takeEvent()) !== null) {
        if (next.kind === 'closed') { result = next.result?.data ?? ''; completion = next.result!.kind; }
      }
      const label = Component.text(`native "${props}" count${state} ${result ? 'result=' + result : 'ready'} completion=${completion}`);
      label.update({ key: 'status' }); label.focus(true, true);
      const row = div().children(Component.text('Left'), Component.text('Right')).build();
      rowStyle.apply(row);
      return div().class('flex-col w-full h-full').children(label, row, editor.element(), dialogs.element()).build();
    },
    event: input => {
      if (eventError && input.type === 'key' && input.key === 'x') throw new Error('intentional event callback failure');
      if (input.type === 'key' && input.key === 'x') component.state++;
      else if (input.type === 'mouse' && input.kind === 'down') component.state++;
      else if (input.type === 'key' && input.key === 'p') component.props = 'omega';
      else if (input.type === 'key' && input.key === 'e') { editor.insert('界é'); component.state = component.state; }
      else if (input.type === 'key' && input.key === 'd') dialogs.open({ kind: 'input', title: 'Name', prompt: 'Enter name' });
      else return false;
      return true;
    },
    dispose: () => { disposed++; },
  });
  const app = new NativeApp(() => component.element());
  if (error || eventError) {
    assert.throws(() => app.run(), /intentional (host|event) callback failure/);
    assert.equal(component.lastError, -12);
  } else app.run();
  app.dispose(); component.dispose(); dialogs.dispose(); editor.dispose(); rowStyle.dispose();
  assert.equal(disposed, 1);
  console.log('ENTRY_POINT_CLEAN_EXIT');
}

initialize();
if (process.argv[2]) host(process.argv[2]); else units();
cleanup();
