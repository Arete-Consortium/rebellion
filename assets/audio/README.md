# Audio Assets

This directory holds optional audio overrides. The game ships with procedural
fallbacks (synthesized via `wav_encoder`); any file dropped here replaces the
fallback at runtime via `AssetServer::load`.

## Music slots

Drop tracks at these paths. `.ogg` is preferred for download size; `.wav` and
`.mp3` work via Bevy's default audio loaders.

| Path | Played during |
|---|---|
| `music/menu.ogg` | Main menu, module select, all menu screens |
| `music/gameplay.ogg` | In-stage gameplay (non-boss) |
| `music/boss.ogg` | Boss fights |
| `music/victory.ogg` | Stage complete sting (one-shot) |
| `music/defeat.ogg` | Game over sting (one-shot) |

## SFX slots

(Coming in next sprint — current SFX are procedural via `wav_encoder`.)

## Asset attribution / IP

This game is a fan project. If you populate the music slots with EVE Online
tracks (e.g. Real X / Permaband, ambient soundtrack tracks), the game becomes
a fan demo using CCP-owned audio. Per CCP's published Fan Content Policy
non-commercial fan projects are generally permitted, but **the policy text
governs**, not this README. Read it before committing to specific tracks:

- CCP Fan Content Policy: https://www.ccpgames.com/legal/fan-content
- EVE Online Fan Kit: https://community.eveonline.com/community/fanart/

When the deployed build includes CCP-owned audio, surface a clear notice in
the UI (e.g. main-menu footer): "Fan project — EVE Online™ assets © CCP Games".

## Loading semantics

- Files are queued for load at startup via `AssetServer::load(path)`
- If the file exists and decodes successfully → file plays
- If the file is missing (404) or fails to decode → procedural fallback plays
- Switching tracks live: replace the file, refresh the page

There is no "music settings" UI to pick which track plays in which slot — the
slot is fixed by filename. To swap tracks, swap the file.
