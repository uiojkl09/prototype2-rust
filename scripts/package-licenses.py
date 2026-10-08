"""Collect resolved crates' distributed notices for the binary package; never read game data."""
import json, pathlib, shutil, sys

metadata = json.load(sys.stdin)
out = pathlib.Path(sys.argv[1]) / 'dependency-licenses'
out.mkdir()
inventory = []
for package in metadata['packages']:
    if package['source'] is None: continue
    source = pathlib.Path(package['manifest_path']).parent
    dest = out / (package['name'] + '-' + package['version'])
    files = [p for p in source.iterdir() if p.is_file() and
             (p.name.upper().startswith(('LICENSE', 'LICENCE', 'COPYING', 'NOTICE')))]
    files += [p for p in source.rglob('*') if p.is_file() and
              p.parent != source and p.name.upper().startswith(('LICENSE','LICENCE','COPYING','NOTICE'))]
    inventory.append({'name': package['name'], 'version': package['version'], 'license': package.get('license'),
                      'repository': package.get('repository'), 'notices': len(files)})
    if files:
        dest.mkdir()
        for file in files:
            relative = file.relative_to(source)
            target = dest / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(file, target)
    elif not package.get('license'):
        raise SystemExit('Dependency lacks license metadata/notices: ' + package['name'])
(out / 'inventory.json').write_text(json.dumps(inventory, indent=2), encoding='utf-8')
missing = [p for p in inventory if p['notices'] == 0]
print(f'Collected notice files for {len(inventory)-len(missing)}/{len(inventory)} resolved crates (includes target/build-only crates)')
if missing:
    print('Crates with SPDX metadata only: ' + ', '.join(p['name'] for p in missing))
