# Controller playtest

Updated September 8, 2026. **Rebellion now requires a controller.** The owner selected this direction for twin-stick movement and directional aiming. Player builds use a fixed controller layout; Options → Controls displays it.

| Action | Xbox-style input |
| --- | --- |
| Move | Left stick; partial deflection gives gentler acceleration |
| Aim | Right stick; returning it to center retains the last aim |
| Fire | Hold RT |
| Hull ability | Press LT once per activation |
| Thrust | Press LB |
| Dodge | Press RB; left-stick direction selects the side |
| Previous / next ammunition | D-pad left / right (autocannon hulls) |
| Interact / confirm | A |
| Overload | Y, when charged |
| Pause / resume | Menu |
| Menu navigation / back | Left stick or D-pad / B |

Aim alone does not fire. A and X no longer fire, RT does not activate an ability, and the D-pad does not steer the ship. Holding LT/LB/RB does not repeat the action when its cooldown ends. Keyboard, mouse and touch cannot navigate or play the shipped game. The operating system's window close controls remain available.

At startup, release the sticks/buttons, then press and release A to continue. If the active controller disconnects, simulation freezes immediately and a connection prompt covers the game. Reconnecting alone does not resume play: release all controls and acknowledge with A again. That acknowledgement is consumed, so it cannot also launch a mission or select a pause-menu action. An existing pause menu remains paused afterward.

One controller remains selected until it disconnects. A second connected controller cannot silently take over. Replacing the selected device requires the same acknowledgement.

## Physical qualification

The game uses Bevy's gamepad reporting. USB-C describes the connector; verify that the operating system actually detects the controller and that the cable carries data. Apple documents supported wireless models and pairing in its [controller instructions](https://support.apple.com/en-us/111101). Successful simulated-input tests do not prove a particular wired or wireless controller is detected.

On each reference Mac/Linux computer, check connection before launch and at the prompt; both sticks and triggers; all four faction choices; pause/resume; unplug while moving and firing; reconnect with a trigger held; then release and acknowledge. Confirm that the ship stays safe during disconnect and that the A acknowledgement does not also select a menu item. Complete a mission and continue from its result screen. Record controller model, connection method, OS and any failure.

The [controller-only checkpoint](CONTROLLER_ONLY_PLAYTEST_2026-09-08.md) records automated and package evidence. Controller detection, stick feel, vibration, sound and a complete human playthrough remain physical acceptance checks.
