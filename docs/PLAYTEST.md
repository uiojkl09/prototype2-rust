# Viewer playtest and diagnostics

This tests the delivered viewer and reader milestone. It is not a character or
gameplay comparison. Keep logs and screenshots locally; do not upload game assets,
extracted data, executable copies, dumps or full installation manifests.

1. Run `prototype2-rust.exe inspect --game <your install>` and compare build/hashes
   with VALIDATION.md. Different identities should be reported rather than assumed
   compatible. No version resource was present in the tested executable/DLL.
2. Run `verify` with the same path. For the tested install, expect 12 archives and
   26,385 entries. This validates indexes. Run `scan --archive cells.rcf` for payloads;
   expect 2,969 P3D files and 811,757 structurally valid chunks.
3. Run `scene`. The default entry should report 38 meshes, 177,596 vertices and
   185,586 triangles; 60 ground collision chunks and 20,075 collision triangles.
4. Run `view`. Expect an untextured interior section in debug colors. Use arrows
   and WASD/QE to inspect walls/floors/roof from different viewpoints. The camera
   starts using bounds and geometric ray probes; it is not a recovered spawn.
5. Press C. Orange surfaces should show the loaded ground collision. Compare
   alignment with visible walls and floors. Some omissions are expected; report
   specific surfaces, location and direction. R resets the camera. Escape exits.
6. Repeat launching/exiting three times and resizing/maximizing the window. Report
   crashes, empty frames or severe stalls. Check your desired 4K/full-screen-sized
   view separately; the default is a 1600x900 logical-pixel window.

For automatic capture and exit (use a path outside the source checkout):

```powershell
.\prototype2-rust.exe view --game 'F:\SteamLibrary\steamapps\common\Prototype 2' --frames 180 --screenshot 'D:\Builds\private-viewer.png'
```

The frame counter is a smoke-test convenience, not a gameplay timing measurement.
Capture requests occur after frame 60; exit waits at least 120 frames when capturing.
For console logs use `scripts/Run.ps1`. It saves under LocalAppData/prototype2-rust/logs.

A useful report includes source commit/build identity, Windows version, GPU/driver,
resolution/DPI, selected entry, exact controls and observed vs expected behavior.
Include command errors and mesh/collision counts. Report screenshots privately.
Movement, powers, missions and audio are not testable yet; do not interpret their
absence as a regression. See MOVEMENT.md for the future original-game comparison.
