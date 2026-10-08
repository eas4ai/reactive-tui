// Preserve a reviewable inventory of every original TypeScript public declaration.
const fs = require('node:fs');
const path = require('node:path');
const { execFileSync } = require('node:child_process');
const ts = require(path.resolve('bindings/typescript/node_modules/typescript'));
const revision = JSON.parse(fs.readFileSync('scripts/abi/baselines/binding-native-baseline.json', 'utf8')).revision;
const destination = 'scripts/abi/baselines/binding-typescript-public-api.json';

function declarations(filename, source) {
  const tree = ts.createSourceFile(filename, source, ts.ScriptTarget.Latest, true);
  const entries = {};
  for (const node of tree.statements) {
    if (ts.isExportDeclaration(node)) {
      entries[node.getText(tree)] = node.getText(tree);
      continue;
    }
    if (!node.modifiers?.some(modifier => modifier.kind === ts.SyntaxKind.ExportKeyword)) continue;
    if (ts.isClassDeclaration(node)) {
      entries[node.name.text] = node.members.filter(member => !member.modifiers?.some(modifier =>
        modifier.kind === ts.SyntaxKind.PrivateKeyword || modifier.kind === ts.SyntaxKind.ProtectedKeyword
      )).map(member => {
        const end = member.body ? member.body.getStart(tree) : member.end;
        return source.slice(member.getStart(tree), end).trim();
      });
    } else if (ts.isVariableStatement(node)) {
      for (const declaration of node.declarationList.declarations) {
        const properties = ts.isObjectLiteralExpression(declaration.initializer)
          ? declaration.initializer.properties.map(property => property.name?.getText(tree)) : [];
        entries[declaration.name.getText(tree)] = {
          type: declaration.type?.getText(tree) ?? 'inferred', properties,
        };
      }
    } else if (node.name) {
      const end = node.body ? node.body.getStart(tree) : node.end;
      entries[node.name.getText(tree)] = source.slice(node.getStart(tree), end).trim();
    }
  }
  return entries;
}

const writing = process.argv[2] === '--write';
const baseline = writing ? null : JSON.parse(fs.readFileSync(destination, 'utf8'));
const files = writing ? execFileSync('git', ['ls-tree', '-r', '--name-only', revision, 'bindings/typescript/src'],
  { encoding: 'utf8', timeout: 30000 }).trim().split('\n').filter(file => file.endsWith('.ts'))
  : Object.keys(baseline.declarations);
const inventory = {};
for (const filename of files) {
  const original = writing ? declarations(filename, execFileSync('git', ['show', `${revision}:${filename}`],
    { encoding: 'utf8', timeout: 30000 }))
    : Object.fromEntries(Object.entries(baseline.declarations[filename]).map(([name, entry]) => [name, entry.previous]));
  const current = fs.existsSync(filename) ? declarations(filename, fs.readFileSync(filename, 'utf8')) : {};
  inventory[filename] = Object.fromEntries(Object.entries(original).map(([name, previous]) => {
    const present = current[name] ?? null;
    const status = present === null ? 'retired' : JSON.stringify(present) === JSON.stringify(previous) ? 'retained' : 'changed';
    return [name, { previous, current: present, status,
      guidance: status === 'retained' ? 'Declaration retained.' : 'See bindings/typescript/MIGRATION.md for the supported native replacement and ownership changes.' }];
  }));
}
const serialized = JSON.stringify({ revision, declarations: inventory }, null, 2) + '\n';
if (writing) fs.writeFileSync(destination, serialized);
else if (fs.readFileSync(destination, 'utf8') !== serialized) {
  throw new Error('Public TypeScript migration inventory is stale; review every changed/retired declaration');
}
console.log(`Public TypeScript declaration inventory reconciled: ${files.length} original modules`);
