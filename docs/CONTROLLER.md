# Xbox controller support — v0.1.1

The Windows viewer reads Xbox-compatible controllers through native XInput 1.4.
No Steam Input profile or keyboard mapper is required. Connect the controller,
launch the viewer, and focus its window. It logs the selected XInput slot (0..3).

| Control | Inspection action |
| --- | --- |
| Left stick | Forward/back and strafe; partial deflection gives slower flight |
| Right stick | Look; upright camera with bounded pitch |
| A / B | Rise / descend |
| LB | Faster flight |
| X | Toggle collision overlay, once per press |
| Y | Reset inspection camera, once per press |
| Menu / Start | Exit, once per press |

These are authored inspection mappings, not recovered Prototype 2 gameplay controls.
Controller-driven Heller movement, action mapping, rumble and rebinding remain future
work. The renderer-independent input layer can supply recorded controller samples
to later simulation; camera timing/speeds must not be used as retail movement rules.

Left/right radial dead zones default to 7849/32767 and 8689/32767, respectively,
using the SDK thumbstick constants as inspection defaults. Magnitude is continuously
rescaled outside the dead zone, with diagonal magnitude capped at one. Negative and
positive raw stick endpoints are normalized symmetrically. Default look speed is
2 radians/second at full deflection. Right stick up looks up unless inverted.

Preferences are optional:

```powershell
.\prototype2-rust.exe view --game 'F:\SteamLibrary\steamapps\common\Prototype 2' --dead-zone 0.25 --look-speed 1.5 --invert-y true
```

`scripts/Run.ps1` also accepts `-DeadZone '0.25' -LookSpeed 1.5 -InvertY` and retains
logs. Dead zone is 0..0.9, look speed is 0.1..10 radians/second; invalid/nonfinite
values fail before opening a window. Use a dot as the CLI decimal separator.

The selected controller stays selected while connected. Other controllers cannot
take over accidentally. Disconnect clears input immediately; new connections are
probed within one second. Held action buttons on connection or focus regain establish
a baseline and must be released/pressed to trigger. Unfocused windows accept no
movement or action input. Keyboard control remains available after disconnection.

For a bounded diagnostic, without game files:

```powershell
.\prototype2-rust.exe controller --seconds 15
```

Each JSON line reports raw controller slots, selected slot, normalized inspection
input and elapsed wall time. This diagnostic reads input even without a focused
viewer, so its action samples are not proof of in-window behavior. A null slot means
no successfully read XInput device in that slot. Keep diagnostic captures locally.

Completed checks: actual XInput slot 0 was detected; observed neutral raw sticks
(-232,159) and (909,129) became zero motion/look. The owner interactively tried the
viewer and reported that controls work with no drift. Automated tests cover analog
magnitude, endpoints/diagonals, held/toggled buttons, focus loss/regain, disconnect,
reconnection, multiple pads and inversion. Those tests do not establish Bluetooth,
wireless adapter, every controller model, or non-Windows support. This backend is
Windows/XInput; devices exposing only another API require separate support.

API/layout/linkage references:
[XInputGetState](https://learn.microsoft.com/en-us/windows/win32/api/xinput/nf-xinput-xinputgetstate),
[XINPUT_GAMEPAD](https://learn.microsoft.com/en-us/windows/win32/api/xinput/ns-xinput-xinput_gamepad),
[XInput versions](https://learn.microsoft.com/en-us/windows/win32/xinput/xinput-versions).
The Windows SDK's Xinput.lib targets the system XINPUT1_4.DLL on this build.
