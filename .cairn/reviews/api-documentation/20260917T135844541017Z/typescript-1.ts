import { div, text } from '@reactive-tui/core';

const root = div().class('flex-col').key('root').child(text('Hello')).build();
try {
  const child = root.getChild(0);
  try { console.log(child.getText()); }
  finally { child.dispose(); }
} finally { root.dispose(); }

export {};
