"""Collect resolved crates' distributed notices for the binary package; never read game data."""
import hashlib, json, pathlib, shutil, sys, urllib.request, urllib.error, urllib.parse
from concurrent.futures import ThreadPoolExecutor

metadata = json.load(sys.stdin)
resolved_ids = {node['id'] for node in metadata['resolve']['nodes']}
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
    candidates = ['LICENSE', 'LICENSE-MIT', 'LICENSE-APACHE', 'LICENSE.md', 'LICENSE.txt',
                  'COPYING', 'NOTICE', 'LICENSE.chromium', 'license/MIT', 'license/APACHE']
    with ThreadPoolExecutor(max_workers=7) as pool:
        files = [item for item in pool.map(fetch, candidates) if item is not None]
    upstream_cache[key] = files
    return files

def declared_license_exception(package, source):
    # These exact releases declare licenses but contain no license/NOTICE files
    # in either their crates or recorded source revisions. Preserve that declaration
    # and distribute the selected standard license; never infer an unknown license.
    exceptions = {
        ('constgebra', '0.1.4'): ('eae8e094e5779f42f1db2a7adf9036aa33744bc8', 'MIT OR Apache-2.0', 'Apache-2.0'),
        ('hexf-parse', '0.2.1'): ('4225763d744183d720f575ae96d04161b4d08ea0', 'CC0-1.0', 'CC0-1.0'),
    }
    exception = exceptions.get((package['name'], package['version']))
    if not exception: return []
    revision, declared, selected = exception
    vcs = json.loads((source / '.cargo_vcs_info.json').read_text())
    if vcs['git']['sha1'] != revision or package.get('license') != declared:
        raise SystemExit('License exception source changed: ' + package['name'])
    address = f'https://raw.githubusercontent.com/spdx/license-list-data/v3.28.0/text/{selected}.txt'
    with urllib.request.urlopen(address, timeout=30) as response: data = response.read(2*1024*1024)
    expected = {'Apache-2.0': '074e6e32c86a4c0ef8b3ed25b721ca23aca83df277cd88106ef7177c354615ff',
                'CC0-1.0': 'a2010f343487d3f7618affe54f789f5487602331c0a8d03f49e9a7c547cf0499'}
    if hashlib.sha256(data).hexdigest() != expected[selected]:
        raise SystemExit('Standard license text changed: ' + selected)
    declaration = (source / 'Cargo.toml.orig').read_bytes()
    return [(f'STANDARD-{selected}.txt', address, data),
            ('NOTICE-license-declaration.txt', package['repository'] + '/tree/' + revision, declaration)]
for package in metadata['packages']:
    if package['source'] is None or package['id'] not in resolved_ids: continue
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
        if not upstream:
            upstream = declared_license_exception(package, source)
            if upstream: entry['license_basis'] = 'Exact crate declaration with selected standard license; upstream has no notice files'
        if not upstream: raise SystemExit('No distribution notices found: ' + package['name'])
        dest.mkdir()
        for name, address, data in upstream: (dest / ('UPSTREAM-' + name.replace('/', '-'))).write_bytes(data)
        entry['notices'] = len(upstream)
        entry['notice_sources'] = [address for name, address, data in upstream]
        entry['notice_sha256'] = {name: hashlib.sha256(data).hexdigest() for name, address, data in upstream}
(out / 'inventory.json').write_text(json.dumps(inventory, indent=2), encoding='utf-8')
missing = [p for p in inventory if p['notices'] == 0]
print(f'Collected notice files for {len(inventory)-len(missing)}/{len(inventory)} resolved Windows crates (includes build-only crates)')
if missing:
    print('Crates with SPDX metadata only: ' + ', '.join(p['name'] for p in missing))
