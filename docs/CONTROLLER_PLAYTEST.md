# Xbox controller on Mac

Updated September 7, 2026. Rebellion uses Bevy's gamepad input on macOS.

Apple lists Bluetooth support for Xbox Wireless Controller model 1708, Xbox Series X/S Wireless Controller, Xbox Elite Wireless Controller Series 2 and Xbox Adaptive Controller. Put the controller in pairing mode, then select it in System Settings → Bluetooth. See [Apple's controller pairing instructions](https://support.apple.com/en-us/111101).

## Default controls

| Action | Xbox input |
| --- | --- |
| Move / menu navigation | Left stick or D-pad |
| Aim and fire | Right stick |
| Fire along the current aim | RT |
| Confirm a menu choice | A |
| Back in menus | B |
| Pause / resume | Menu or View |

The right trigger accepts both axis and analog-button reporting. Input is sampled after Bevy processes the current frame's events. Disconnect clears all held input; reconnect detects the controller again. Keyboard and pointer controls remain available during this playtest.

## Verification boundary

Automated tests exercise analog-trigger reporting and held-input clearing on disconnect. Menu route tests use the shared input resource. These checks do not prove a physical Bluetooth controller's mapping, wireless stability, rumble or comfort.

Physical qualification should cover: connect before launch; connect at the menu; select either chapter and side; move and shoot with RT and the right stick; pause/resume; disconnect while holding fire; reconnect; complete a mission and continue from the result screen. Check that releasing the stick stops menu movement and that B never confirms a choice. Record the controller model and macOS version with any failure.
