const assert = require('node:assert/strict');
const path = require('node:path');
const { execFileSync } = require('node:child_process');
const terminalMode = () => execFileSync('stty', ['-g'], {
  stdio: ['inherit', 'pipe', 'pipe'], encoding: 'utf8', timeout: 5000,
});
const beforeSignals = ['SIGINT', 'SIGTERM', 'exit'].map(name => process.listenerCount(name));
const sdk = require(path.join(__dirname, '..', 'dist'));
const { output, koffi } = require(path.join(__dirname, '..', 'dist', 'ffi'));
assert.deepEqual(['SIGINT', 'SIGTERM', 'exit'].map(name => process.listenerCount(name)), beforeSignals);

sdk.initialize();
try {
  const version = sdk.getVersion();
  assert.equal(version.abiVersion, 1);
  assert.ok(Object.values(version).every(Number.isInteger));
  assert.equal(sdk.lib.rtui_terminal_create(null), sdk.ReactiveError.NullPointer);
  assert.equal(sdk.lib.rtui_surface_create(2, 2, null), sdk.ReactiveError.NullPointer);
  assert.equal(sdk.lib.rtui_element_get_type(null, output('RTuiElementType')), sdk.ReactiveError.NullPointer);
  const signal = sdk.lib.rtui_signal_new_string('owned signal 界');
  assert.ok(signal);
  const nativeFreeString = sdk.lib.rtui_string_free;
  let releases = 0;
  sdk.lib.rtui_string_free = pointer => { releases++; nativeFreeString(pointer); };
  try {
    const disposed = koffi.stats().disposed;
    assert.equal(sdk.lib.rtui_signal_get_string_owned(signal), 'owned signal 界');
    assert.equal(koffi.stats().disposed, disposed + 1, 'Owned C string must be released after conversion');
    assert.equal(releases, 1, 'The Rust string deallocator must run exactly once');
  } finally {
    sdk.lib.rtui_string_free = nativeFreeString;
    sdk.lib.rtui_signal_destroy_new(signal);
  }

  const terminal = new sdk.Terminal();
  const originalMode = terminalMode();
  try {
    assert.deepEqual(terminal.getSize(), { width: 80, height: 24 });
    const caps = terminal.getCapabilities();
    assert.equal(Object.keys(caps).length, 10);
    assert.equal(caps.mouse, true);
    assert.ok(caps.unicode_level === 1 || caps.unicode_level === 2);
    for (const [name, value] of Object.entries(caps)) {
      if (name !== 'unicode_level') assert.equal(typeof value, 'boolean');
    }
    terminal.init();
    assert.notEqual(terminalMode(), originalMode, 'Native setup must change terminal mode');
    terminal.clear(); terminal.setCursor(2, 3);
    terminal.hideCursor(); terminal.showCursor();
    assert.throws(() => terminal.setCursor(0, 1), RangeError);
  } finally { terminal.dispose(); terminal.dispose(); }
  assert.equal(terminalMode(), originalMode, 'Disposal must restore terminal mode');
  assert.throws(() => terminal.getSize(), /disposed/);

  assert.throws(() => new sdk.Surface(0, 2), sdk.ReactiveTUIError);
  assert.throws(() => new sdk.Surface(-1, 2), RangeError);
  const surface = new sdk.Surface(4, 3);
  try {
    surface.setCell(3, 2, '界', 0x123456, 0xABCDEF);
    assert.deepEqual(surface.getCell(3, 2), { content: '界', fg: 0x123456, bg: 0xABCDEF });
    surface.setCell(0, 0, '😀'); assert.equal(surface.getCell(0, 0).content, '😀');
    assert.throws(() => surface.setCell(0, 0, 'e\u0301'), TypeError);
    assert.throws(() => surface.setCell(0, 0, '\uD800'), TypeError);
    assert.throws(() => surface.getCell(4, 0), RangeError);
    assert.throws(() => surface.clear(256), RangeError);
    surface.clear(0x12, 0x34, 0x56);
    assert.equal(surface.getCell(3, 2).bg, 0x123456);
    surface.resize(7, 2); assert.deepEqual(surface.getSize(), { width: 7, height: 2 });
  } finally { surface.dispose(); surface.dispose(); }
  assert.throws(() => surface.getSize(), /disposed/);

  const renderer = new sdk.Renderer(4, 3);
  let borrowed;
  try {
    borrowed = renderer.getSurface();
    assert.deepEqual(renderer.getSize(), { width: 4, height: 3 });
    renderer.clear(0x12, 0x34, 0x56);
    assert.equal(borrowed.getCell(0, 0).bg, 0x123456);
    assert.throws(() => borrowed.resize(2, 2), /owning renderer/);
    renderer.resize(6, 2);
    assert.throws(() => borrowed.getSize(), /expired/);
    borrowed.dispose();
    borrowed = renderer.getSurface();
    assert.deepEqual(borrowed.getSize(), { width: 6, height: 2 });
    renderer.frame(() => borrowed.setCell(1, 1, 'X', 0xFFFFFF, 0));
    assert.equal(borrowed.getCell(1, 1).content, 'X');
  } finally { renderer.dispose(); renderer.dispose(); }
  assert.throws(() => borrowed.getSize(), /disposed/);
  borrowed.dispose();

  const child = sdk.text('owned child 界');
  const builder = sdk.div().class('flex-row').key('root').child(child);
  assert.throws(() => child.getType(), /consumed/);
  const parent = builder.build();
  let clone;
  try {
    assert.equal(parent.getType(), sdk.ElementType.Layout);
    assert.equal(parent.getClass(), 'flex-row'); assert.equal(parent.getKey(), 'root');
    assert.equal(parent.getChildCount(), 1);
    assert.throws(() => parent.getChild(1), sdk.ReactiveTUIError);
    assert.throws(() => parent.addChild(parent), /itself/);
    clone = parent.getChild(0);
    assert.equal(clone.getText(), 'owned child 界');
    assert.throws(() => builder.build(), /consumed/);
  } finally { parent.dispose(); builder.dispose(); child.dispose(); }
  assert.equal(clone.getText(), 'owned child 界'); clone.dispose();

  const component = new sdk.Component('example', { key: 'id', class: 'p-2' });
  try {
    assert.equal(component.getName(), 'example'); assert.equal(component.getKey(), 'id');
    component.update({ key: 'next' }); assert.equal(component.getKey(), 'next');
    assert.throws(() => component.update({ key: 'bad\0key' }), /NUL/);
    assert.throws(() => component.update({ state: 1 }), /Unsupported/);
  } finally { component.dispose(); }
  const abandoned = sdk.div();
  assert.throws(() => abandoned.child(abandoned), /itself/); abandoned.dispose();
  const button = sdk.builders.primaryButton('Go').build();
  try { assert.equal(button.getText(), 'Go'); assert.match(button.getClass(), /bg-blue-500/); }
  finally { button.dispose(); }
} finally { sdk.cleanup(); }
console.log('TypeScript native consumer lifecycle, values, null errors and ownership checks passed');
