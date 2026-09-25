import { initialize, cleanup, div, text } from '../src';

initialize();
try {
  const root = div().key('root').class('flex-col p-2').child(text('Native element')).build();
  try {
    const child = root.getChild(0); // An owned clone.
    try { console.log(root.getKey(), child.getText()); }
    finally { child.dispose(); }
  } finally { root.dispose(); }
} finally { cleanup(); }
