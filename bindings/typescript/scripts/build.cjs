// Rebuild generated output so retired modules cannot survive in a package tarball.
const fs = require('node:fs');
const path = require('node:path');
const { spawnSync } = require('node:child_process');
const root = path.join(__dirname, '..');
fs.rmSync(path.join(root, 'dist'), { recursive: true, force: true });
const result = spawnSync(process.execPath, [require.resolve('typescript/bin/tsc')], {
  cwd: root, stdio: 'inherit', timeout: 60000,
});
if (result.error) throw result.error;
process.exit(result.status ?? 1);
