"""Audit the Git index allowlist and text contents before publishing. No game data is read."""
import pathlib, re, subprocess, sys

root = pathlib.Path(__file__).resolve().parents[1]
def git(*args):
    return subprocess.check_output(['git', '-C', str(root), *args])
exact = {'.gitignore', '.gitattributes', 'AGENTS.md', 'README.md', 'STATUS.md', 'LICENSE',
         'THIRD_PARTY.md', 'Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml', '.github/workflows/ci.yml'}
def allowed(path):
    p = pathlib.PurePosixPath(path)
    if '..' in p.parts or p.is_absolute(): return False
    return path in exact or (p.parts[0] in ('src', 'tests') and p.suffix == '.rs') or \
        (p.parts[0] == 'docs' and p.suffix == '.md') or \
        (len(p.parts) == 2 and p.parts[0] == 'scripts' and p.suffix in ('.py', '.ps1'))
bad_patterns = [
    re.compile(r'gh[pousr]_[A-Za-z0-9]{20,}'),
    re.compile(r'github_pat_[A-Za-z0-9_]{20,}'),
    re.compile(r'AKIA[A-Z0-9]{16}'),
    re.compile(r'-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----'),
    re.compile(r'(?:C:[\\/]Users[\\/]|/[h]ome/)[^\s<>]+', re.I),
    re.compile(r'\b(?:FUN|DAT)_[0-9a-fA-F]{6,}\b'),
]
errors = []
entries = [x for x in git('ls-files', '--stage', '-z').decode().split('\0') if x]
if not entries: errors.append('Git index is empty')
seen = set()
for row in entries:
    metadata, path = row.split('\t', 1)
    mode, oid, stage = metadata.split()
    if mode not in ('100644', '100755') or stage != '0': errors.append('Non-regular or conflicted index entry: ' + path)
    if not allowed(path): errors.append('Path is outside publication allowlist: ' + path)
    if path.casefold() in seen: errors.append('Case-insensitive path collision: ' + path)
    seen.add(path.casefold())
    data = git('show', ':' + path)
    if len(data) > 512 * 1024: errors.append('Unexpectedly large source file: ' + path)
    if b'\0' in data: errors.append('Binary data in text file: ' + path)
    try: content = data.decode('utf-8')
    except UnicodeDecodeError:
        errors.append('Invalid UTF-8: ' + path); continue
    if any(p.search(content) for p in bad_patterns): errors.append('Sensitive/private or decompiler-shaped content: ' + path)
if errors:
    print('\n'.join(errors)); sys.exit(1)
print(f'PASS: {len(entries)} indexed text files allowed; no binary, detected credentials, personal home paths or decompiler placeholders')
print('This heuristic audit does not establish provenance or completeness. Review the full staged diff as well.')
