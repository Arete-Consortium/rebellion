# Audio direction and assets

The September 8 combat pass uses original procedural synthesis for a weighty industrial space-combat sound. It layers mechanical attacks, capacitor tones, filtered noise and bass resonance. No EVE recordings are included in these effects.

| Cue | Current treatment |
| --- | --- |
| Autocannon / artillery | Short mechanical crack with a falling low-frequency body |
| Laser | Capacitor chirp with a resonant energy tail |
| Railgun | Sharp electromagnetic discharge and metallic ringing |
| Missile | Ignition transient followed by filtered thrust noise |
| Drone | Compact modulated electronic burst |
| Carrier arrival | 2.2-second charge, warp collapse and low arrival pulse |
| Explosions | Three sizes of bass rumble, transient noise and metallic debris |
| Shield / armor / hull impacts | Electrical resonance, metallic ring and structural crunch |
| Health warnings | Distinct pulse patterns; critical hull warning interrupts lower-priority alarms |

Combat effects are generated once at startup in `src/systems/audio/generators.rs` and reused. Weapon playback has a maximum of eight active voices and two new voices per frame. Ordinary explosions have eight active voices and three new voices per frame, with one separate reserved boss-explosion voice. New cues have short edge fades and headroom; this is not a master-bus limiter. Dense combat still needs listening and mix adjustment on speakers and headphones. Ability, pickup and UI cues retain the previous synthesis for a subsequent pass. Music follows the combat state for both chapters and plays a single result sting when a mission ends.

To export the actual generated combat cues for listening:

```sh
REBELLION_AUDIO_PREVIEW_DIR=build/audio-review cargo test --locked --lib combat_cues_have_headroom_distinct_timbres_and_clean_edges
```

This writes individual WAV files plus `preview.wav` in this order: autocannon, laser, railgun, missile, drone, carrier warp, heavy explosion, three impacts, then three health warnings. These review files are not shipped in the game assets.

## Music and imported audio

All shipped music is generated locally: menu ambience, gameplay, boss combat, victory and defeat. The unused automatic requests for five absent Ogg files were removed, along with their file-override handles and selection code. This prevents the previous missing-file errors and avoids advertising an Ogg decoder that this build does not enable.

Dropping WAV/Ogg/MP3 files in this directory does not replace music or effects. A future imported-music pass should add only the chosen supported format, loading/error handling and source/license records when actual tracks are selected. No external recordings are required for the current free community candidate.
