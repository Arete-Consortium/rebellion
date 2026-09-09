# Controller playtest

Updated September 8, 2026. **Rebellion now requires a controller.** The owner selected this direction for twin-stick movement and directional aiming. Player builds use a fixed controller layout; Options → Controls displays it.

| Action | Xbox-style input |
| --- | --- |
| Move | Left stick; partial deflection gives gentler acceleration |
| Aim and fire | Push right stick toward the target; center it to stop firing |
| Hull ability | Press LT once per activation |
| Thrust | Press RT |
| Dodge | Press RB; left-stick direction selects the side |
| Previous / next ammunition | X / B or D-pad left / right (autocannon hulls) |
| Select timed booster | D-pad up/down; skips empty slots |
| Use selected booster | Y; one dose per fresh press |
| Interact / confirm | A |
| Overload | LB, when charged |
| Pause / resume | Menu |
| Menu navigation / back | Left stick or D-pad / B |

Right-stick deflection fires in the aimed direction at the weapon's normal cadence. Centering the stick stops primary fire while retaining the last aim for special bursts. Small centered-stick noise stays inside the deadzone. RT thrusts; LT uses the hull ability. Holding a trigger or bumper does not repeat its action when the cooldown ends. X/B cycle ammunition once per press, and the D-pad does not steer the ship. Timed boosters are stored: D-pad up/down selects, Y activates. Carry up to three doses each of Overclocker, Pyrolancea and X-Instinct. An empty or already-active selection spends nothing. Extra pickups stay in space while their slot is full. Unused doses carry between missions; death, a fresh hull/run selection or an explicit restart clears them. Repairs and capacitor/cooling pickups remain instant. Keyboard, mouse and touch cannot navigate or play the shipped game. The operating system's window close controls remain available.

At startup, release the sticks/buttons, then press and release A to continue. If the active controller disconnects, simulation freezes immediately and a connection prompt covers the game. Reconnecting alone does not resume play: release all controls and acknowledge with A again. That acknowledgement is consumed, so it cannot also launch a mission or select a pause-menu action. An existing pause menu remains paused afterward.

One controller remains selected until it disconnects. A second connected controller cannot silently take over. Replacing the selected device requires the same acknowledgement.

## Physical qualification

The game uses Bevy's gamepad reporting. USB-C describes the connector; verify that the operating system actually detects the controller and that the cable carries data. Apple documents supported wireless models and pairing in its [controller instructions](https://support.apple.com/en-us/111101). Successful simulated-input tests do not prove a particular wired or wireless controller is detected.

On each reference Mac/Linux computer, check connection before launch and at the prompt; both sticks and triggers; all four faction choices; pause/resume; unplug while moving and firing; reconnect with the right stick or a trigger held; then release and acknowledge. Confirm that the ship stays safe during disconnect and that the A acknowledgement does not also select a menu item. Complete a mission and continue from its result screen. Record controller model, connection method, OS and any failure.

Collect a timed booster and confirm it remains stored until Y is pressed. Select a dose with D-pad up/down while moving and aiming, activate it, and check the countdown and sound. Press Y again during the effect and hold it through expiration: neither should consume another dose. Fill a drug slot, confirm excess pickups remain in space, then use a dose and collect one. Carry unused doses into the next mission; confirm an explicit restart clears them. Judge whether moving a thumb away from the right stick to press Y feels comfortable under pressure.

The [manual booster checkpoint](MANUAL_BOOSTER_PLAYTEST_2026-09-08.md) records automated and package evidence. Controller detection, stick feel, vibration, sound and a complete human playthrough remain physical acceptance checks.
