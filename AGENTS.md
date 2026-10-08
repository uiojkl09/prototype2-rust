# prototype2-rust working instructions

Read README.md, STATUS.md, docs/FORMATS.md, docs/RESEARCH.md and docs/VALIDATION.md first.

The objective is an independent, faithful Prototype 2 runtime. Current code is a
data reader and experimental section viewer. Maintain the distinction. A successful
build is not gameplay parity. Do not replace blocked work with invented mechanics.

Work directly with the user. Do not spawn subagents or delegate unless the user
explicitly requests it. Continue authorized research, implementation and verification.

Use the owner's installed game read-only. Never modify a retail executable, install
compatibility patches, change CPU affinity, or run the retail engine from this runtime.
No assets, proprietary/decompiled source, dumps, databases, credentials or personal
files may enter this Git repository. Keep private investigation in a sibling directory
outside the repository. Do not copy files out of the neighboring modding repository.

The default-deny .gitignore and scripts/audit-publication.py are mandatory before
staging/pushing. Never force-add ignored files. Review the complete staged diff, then
audit the tracked contents, including new files and anything going into a release.
Publication of this project's original source is authorized by the owner.

Record evidence with game build, binary hashes, entry name, chunk offset/ID and
measured outcome. Label confirmed structures, interpretations, hypotheses and unknowns.
Keep public notes about formats/behavior. Do not publish decompiler output or private
Discord messages. Preserve third-party licenses/attribution for any code adapted.

Format readers and simulation logic stay independent of Bevy. Unknown layouts fail
with context; do not silently fake missing assets. Retain reader limits and malformed
input tests. Future simulation must have an explicit fixed tick, input/state snapshots
and repeatable replay. Choose tick rate and mechanics from evidence, not convenience.

Run cargo fmt --all --check, cargo test --locked --no-default-features and cargo clippy
--locked --all-targets -- -D warnings. Run opt-in retail tests when data is available.
Use scripts/Build.ps1 and scripts/Package.ps1 for deliverables; each build includes
source, build/test instructions, notices and honest limitations. Visually inspect the
viewer; keep screenshots and logs private. Maintain STATUS.md after every milestone.
