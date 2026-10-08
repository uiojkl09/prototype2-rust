"""Collect resolved crates' distributed notices for the binary package; never read game data."""
import json, pathlib, shutil, sys, urllib.request, urllib.error, urllib.parse
from concurrent.futures import ThreadPoolExecutor

metadata = json.load(sys.stdin)
out = pathlib.Path(sys.argv[1]) / 'dependency-licenses'
out.mkdir()
inventory = []
upstream_cache = {}
def upstream_notices(package, source):
    vcs_file = source / '.cargo_vcs_info.json'
    repository = package.get('repository') or ''
    url = urllib.parse.urlparse(repository)
    if not vcs_file.exists() or url.hostname != 'github.com': return []
    revision = json.loads(vcs_file.read_text())['git']['sha1']
    parts = url.path.strip('/').removesuffix('.git').split('/')[:2]
    if len(parts) != 2: return []
    repo = '/'.join(parts)
    key = (repo, revision)
    if key in upstream_cache: return upstream_cache[key]
    def fetch(name):
        address = f'https://raw.githubusercontent.com/{repo}/{revision}/{name}'
        try:
            request = urllib.request.Request(address, headers={'User-Agent': 'prototype2-rust-license-packager'})
            with urllib.request.urlopen(request, timeout=30) as response: data = response.read(2*1024*1024)
            return (name, address, data)
        except urllib.error.HTTPError as error:
            if error.code == 404: return None
            raise
    candidates = ['LICENSE', 'LICENSE-MIT', 'LICENSE-APACHE', 'LICENSE.md', 'LICENSE.txt', 'COPYING', 'NOTICE']
    with ThreadPoolExecutor(max_workers=7) as pool:
        files = [item for item in pool.map(fetch, candidates) if item is not None]
    upstream_cache[key] = files
    return files
for package in metadata['packages']:
    if package['source'] is None: continue
    source = pathlib.Path(package['manifest_path']).parent
    dest = out / (package['name'] + '-' + package['version'])
    files = [p for p in source.iterdir() if p.is_file() and
             (p.name.upper().startswith(('LICENSE', 'LICENCE', 'COPYING', 'NOTICE')))]
    files += [p for p in source.rglob('*') if p.is_file() and
              p.parent != source and p.name.upper().startswith(('LICENSE','LICENCE','COPYING','NOTICE'))]
    entry = {'name': package['name'], 'version': package['version'], 'license': package.get('license'),
             'repository': package.get('repository'), 'notices': len(files)}
    inventory.append(entry)
    if files:
        dest.mkdir()
        for file in files:
            relative = file.relative_to(source)
            target = dest / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(file, target)
    else:
        upstream = upstream_notices(package, source)
        if not upstream: raise SystemExit('No distribution notices found: ' + package['name'])
        dest.mkdir()
        for name, address, data in upstream: (dest / ('UPSTREAM-' + name)).write_bytes(data)
        entry['notices'] = len(upstream)
        entry['notice_sources'] = [address for name, address, data in upstream]
(out / 'inventory.json').write_text(json.dumps(inventory, indent=2), encoding='utf-8')
missing = [p for p in inventory if p['notices'] == 0]
print(f'Collected notice files for {len(inventory)-len(missing)}/{len(inventory)} resolved crates (includes target/build-only crates)')
if missing:
    print('Crates with SPDX metadata only: ' + ', '.join(p['name'] for p in missing))
