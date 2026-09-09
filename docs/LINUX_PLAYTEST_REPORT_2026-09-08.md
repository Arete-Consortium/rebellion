# Ubuntu / Linux playtest checkpoint — September 8

This preserves the first Linux candidate. The [controller-only checkpoint](CONTROLLER_ONLY_PLAYTEST_2026-09-08.md) records the current input direction and candidates; the [hull ability checkpoint](HULL_ABILITY_PLAYTEST_2026-09-08.md) preserves the intermediate combat changes and validation.

The Intel/AMD **x86_64** candidate passed the [Ubuntu 22.04 workflow](https://github.com/Arete-Consortium/rebellion/actions/runs/34287059762).

- [Download Rebellion-linux-x86_64](https://github.com/Arete-Consortium/rebellion/actions/runs/34287059762/artifacts/10080722928) — 35,874,979 bytes (about 36 MB). GitHub sign-in is required; this Actions artifact expires October 8, 2026.
- Build source: `00e2ec281e69ca660879322c16966157d6345349`, on `codex/linux-playtest`; Rust 1.98.1, target `x86_64-unknown-linux-gnu`.
- The 327 game inputs from the verified Mac checkpoint match both the local files and the published Git tree. Gameplay, hull art and approved booster art are unchanged.

## Verified

**539 tests passed**, zero failed or ignored, across 37 test summaries. The optimized release then built successfully. A fresh extraction passed its file checksums and shared-library check. Its real executable launched from an unrelated directory with an isolated muted save, initialized and remained alive for at least ten seconds, with no panic or missing-asset error detected.

The [startup evidence](https://github.com/Arete-Consortium/rebellion/actions/runs/34287059762/artifacts/10080723585) contains the log, resolved libraries and machine-readable result. CI used Xvfb and software Vulkan; it does not establish hardware performance, sound quality, controller detection or a human campaign playthrough.

## Play on the other PC

Extract the downloaded ZIP, then run these commands in its folder:

```sh
sha256sum --check Rebellion-linux-x86_64.tar.gz.sha256
tar -xzf Rebellion-linux-x86_64.tar.gz
./Rebellion-linux-x86_64/play.sh
```

The archive includes the game, assets, launcher, license, build information and internal checksums. See [Linux setup and troubleshooting](LINUX_PLAYTEST.md) for runtime libraries, isolated saves and source builds. Generated executables remain in Actions artifacts and ignored `dist/` output.

First qualify the complete Caldari/Gallente three-mission route on the physical PC. Record the distribution, GPU, selected hull, sound/controller behavior and booster readability. The RBL-14 runtime gaps identified at this checkpoint are covered by the [hull ability follow-up](HULL_ABILITY_PLAYTEST_2026-09-08.md); the [controller-only follow-up](CONTROLLER_ONLY_PLAYTEST_2026-09-08.md) implements the subsequent input decision.

This is a testing-branch candidate. It does not promote the game to a public release or change `main`.
