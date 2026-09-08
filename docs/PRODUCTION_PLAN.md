# Rebellion improvement and production plan

Prepared 2026-09-07. Status: proposed production roadmap for the existing
`codex/finishability-pass` working tree, based on source revision `226f3bc` and
the local repairs. This is the active planning document; the
[repair baseline](COMPLETION_PLAN.md) preserves completed work and verification.

Latest playable checkpoint: [booster combat fixes and campaign probes](BOOSTER_COMBAT_PLAYTEST_2026-09-08.md) connects speed/damage drugs to actual movement and firing, repairs temporary ability speed, resets refreshed HUD warnings, and fixes a ship/scenery collision crash. Both starter factions reached all three CG mission results with ordinary scripted input and default parallel scheduling. There are 539 passing tests; human balance and controller qualification remain open. Remaining hull ability effects are the next code priority.

Latest presentation follow-up: [classic booster correction](CLASSIC_BOOSTER_PLAYTEST_2026-09-08.md) replaces shared industrial artwork with archived finished-drug bottles, prioritizes the classic families, covers remaining consumables with newer boosters, and makes the previously unreachable Mindflood pickup available in normal drops. The [first booster/HUD checkpoint](BOOSTER_PLAYTEST_2026-09-08.md) is preserved as history.

Encounter follow-up: [Hedion transport encounter](TRANSPORT_PLAYTEST_2026-09-08.md) implements mission-2 boarding/convoy objectives for both Elder Fleet sides, a bounded transparent invulnerability outline and objective-failure UX. Human encounter tuning and the wider release gates remain open.

Previous follow-up: [September 8 save recovery and combat audio](PLAYTEST_2026-09-08.md) implements native atomic writes/backups, damaged-file preservation, quieter startup, impact/warning synthesis and corrected music transitions. The controller hardware check remains separate.

Current checkpoint: the owner confirmed **Play → chapter → faction → difficulty → hull → mission**, with Minmatar/Amarr and Caldari/Gallente as the two exposed conflicts. Each side uses its own hull roster. The Mac candidate, ship import and route evidence are recorded in [MAC_PLAYTEST_REPORT.md](MAC_PLAYTEST_REPORT.md). Earlier 506-test and asset-size measurements in the repair baseline describe the previous build. M0 public release qualification remains open until human playtests and distribution checks pass.

The owner confirmed on 2026-09-07 that Rebellion is a free portfolio and community project. Carrier-led wave arrivals, top-down sprites and a readable background conflict are part of the intended experience.

## Product target and scope

Build an immediately playable, controller-friendly EVE arcade shooter in
which ship choice changes how the player survives and attacks. The first
release should demonstrate responsive combat, readable danger, distinctive
faction weapons, satisfying bosses and a reason to replay.

All playable releases and future campaigns will be free. Success means
people can enjoy the game, understand the engineering and design work behind
it, and contribute useful feedback or improvements. Paid content and
monetization work are outside this production plan.

**Current deliverable: a Mac playtest with both conflict chapters, and a complete three-mission Caldari–Gallente slice.** Keep
the existing Rust/Bevy foundation. Qualify the current Mac first; add Windows
qualification before claiming a Windows release. Keep other native platforms
and the browser as explicit qualification milestones. The Elder Fleet conflict exposes its existing faction-specific campaign for playtesting. Nightmare, survival and other modes stay outside the main chapter selection while these routes are qualified.

| Include in the first public candidate | Hold for later production |
| --- | --- |
| Both conflict choices; three CG missions and their boss encounters | Further campaigns, Nightmare and survival-mode releases |
| Two faction perspectives and a small, validated starter roster | More hulls, T3 unlocks and large upgrade trees |
| Keyboard and physical controller support throughout the full route | Mobile/touch qualification and platform-specific store integrations |
| Reliable mission progress, settings and local best scores | Online accounts, multiplayer and online leaderboards |
| Consistent ship sprites, legible effects, coherent audio and results | Full 3D conversion, engine migration and a new content-editor framework |

The public candidate should expose only qualified routes. Public menu scope
is a build/content choice, not a reason to delete implemented campaigns.

Confirmed direction and decisions to settle at the first milestone:

| Decision | Proposed direction | Evidence still needed |
| --- | --- | --- |
| Release model | **Confirmed: free portfolio and community project, including future campaigns** | Owner's direction recorded in [the project decision note](../sessions/2026-09-07-community-project-direction.md); retain asset sources and credits |
| Reference platform | Current Mac for development and initial qualification | Exact supported OS, device, display/resolution and controller; Windows test machine if included |
| Run length | Test the current 8–12-minute successful-run target | Timed complete runs; record learning time, menus and retries separately |
| Starter roster | Three immediately available frigates per faction; 22 hull definitions across four factions | Human feel testing of roles, unlocks and final bow/stern qualification |
| Progression | Mission continuation and local bests, with explicit reset rules | Resolve actual upgrade/unlock behavior against menu promises |

The older [slice definition](VERTICAL_SLICE.md) describes 20–30 minutes and
different wave, ship and boss values from current source. Preserve that
historical document. After a scope decision, record the decision through its
existing decision-log process and append a reconciliation note. Proposed
targets here do not silently change gameplay or approve a new roster.

## Milestones, dependencies and exit gates

Indicative planning allowance: **6–10 full-time-equivalent workweeks** for
the polished demo, assuming substantial reuse of existing content and access
to an artist/reviewer, physical controllers and playtesters. This is a low
confidence planning range, not a delivery commitment. Start with a
discovery-and-repair sprint sized to the owner's available time; re-estimate after a complete packaged
run and one approved art sample. Wider campaign production is additional.
External permission/review waits and part-time availability extend elapsed time.

| Milestone | Work and player outcome | Exit gate | Dependency |
| --- | --- | --- | --- |
| M0 — Qualified baseline | Reconcile the release contract; make a native candidate runnable outside the checkout; capture the whole current route | Named platform and offered roster; clean-save package launches offline from an arbitrary working directory; one recorded run with ranked blockers | Existing repair baseline |
| M1 — Complete and reliable | Repair advertised inputs, saves, progression and truthful results; close all slice softlocks | Every offered starter completes missions 1–3; pause/death/retry/quit/relaunch matrix passes; no crash, lost save or blocked continuation | M0 |
| M2 — Combat worth replaying | Tune movement/aim, weapon roles, enemy encounters, boss patterns, scoring and retry | Five observed testers; proposed target of four understanding basic controls without a manual and four completing Newbro by their third attempt; testers can explain ship tradeoffs and their deaths | Stable M1 route |
| M3 — Consistent presentation | Approve one art standard, then finish slice hulls, backgrounds, effects, HUD and audio | Every used hull passes identity/crop/orientation/pivot/hardpoint review; threats remain legible with reduced effects, muted audio and color removed | Art sample can start during M1; final acceptance uses M2 encounters |
| M4 — Measured performance and packages | Fix known performance defects; profile real combat; qualify native artifacts and reproducible build inputs | Release-build timing evidence on each claimed device; complete playthrough from extracted candidate; package/asset hashes and checks tied to the same revision | M1; repeat affected windows after M2/M3 changes |
| M5 — Community and portfolio release | Run external acceptance, resolve blockers, prepare the playable release, gameplay capture, engineering case study and contributor entry points | Two consecutive candidate builds pass the release checklist below; portfolio claims link to evidence; owner approves the specific tested artifact for publication | M2–M4 and distribution readiness |

The critical path is packaged baseline → complete route → combat iteration →
finished presentation → candidate acceptance. Art sampling and profiling
infrastructure can proceed alongside repairs. Expand one additional campaign
only after the demo meets its gates, applying the same process to that campaign.

## Improvements that define quality

| Area | Concrete improvement | Acceptance evidence |
| --- | --- | --- |
| Movement and aiming | Tune response, acceleration, dead zones and aim behavior on the actual starter ships; preserve clear control under damage and effects | Captured keyboard/controller sessions; no stuck input or loss of control on focus loss/reconnect; record latency observations rather than claiming an unmeasured number |
| Faction and ship identity | Give Caldari ranged/shield-oriented roles and Gallente close-range/drone-oriented roles; distinguish roles within each faction | Each offered ship has an understandable advantage and cost; no starter consistently loses on both survival and completion time in the pilot cohort |
| Encounter teaching | Mission 1 teaches move/fire/defense; Mission 2 adds range and pattern reading; Mission 3 combines those skills | First-time players explain what each new threat asks them to do; increasing pressure comes from combinations and positioning as well as enemy counts |
| Bosses | Create tactical phase changes, warning cues, dodge routes and recovery windows | Prototype a 0.6-second warning for heavy attacks, then tune from observed response time; recorded deaths reveal the source and a plausible avoidance response |
| Scoring and retry | Track actual maximum chain, completion time and local personal bests; make results suggest one improvement goal | Correct statistics after an expiring combo; proposed retry-to-control target under five seconds with the chosen loadout retained |
| Readability | Give hostile fire a consistent shape/brightness language; subordinate scenery and friendly effects; align visible hulls and hitboxes | Dense combat captures at gameplay scale; no reliance on color or sound alone; low-effects settings retain essential warnings |
| Onboarding and accessibility | Brief in-play teaching, accurate button prompts, working remaps, independent audio/effect controls and reliable pause | Proposed clean-save target: controllable combat within 30 seconds; full keyboard/controller route with reduced shake and muted audio |
| Audio and presentation | Distinguish weapons, heavy threats, damage and rewards; reconcile optional music loading with supported formats | No missing required asset errors; generated tracks follow boss/results transitions; hearing review at ordinary and crowded combat levels |

Prototype and compare small changes. Keep one recorded baseline encounter,
change one major combat variable at a time, and retain the better result.
Collect deaths, completion times, retry choices, quit points and short player
explanations. Small-cohort targets guide iteration; they are not market-demand
evidence. Include EVE players, experienced shooter players and a newcomer.

## Engineering and materials work queue

All items below are planned, not implemented by this planning pass. Priorities
apply to the first demo; dependencies prevent broad refactors from delaying it.

| ID / priority | Task and existing evidence | Done when |
| --- | --- | --- |
| RBL-01 / P0 | Reconcile the release contract: current mission definitions, entry flow, roster, duration, upgrades and save promises disagree with older docs | One owner-approved table agrees with the offered game route; unsupported public choices are absent or accurately labelled |
| RBL-02 / P0 | Apple Silicon Mac and Ubuntu 22.04 x86_64 candidates are packaged and pass clean-extraction launch checks; see the [Linux checkpoint](LINUX_PLAYTEST_REPORT_2026-09-08.md) | Automated packaging checks pass. Qualify sound, physical controller behavior, hardware performance and a complete packaged human run on the reference computers |
| RBL-03 / P0 | Implemented for the Mac candidate: atomic native saves, one valid backup, recovery and damaged-primary preservation; see September 8 evidence | Mac regressions pass. Complete cross-platform qualification and add a visible recovery/write-failure notice before public release |
| RBL-04 / P0 | Exercise the whole CG route and its persistence promises; the headless suite does not prove it | Starter/mission/boss transition matrix passes, with a regression for every reproduced blocker and a recorded packaged run |
| RBL-05 / P1 | Finish offered menu/pause remappings at their runtime consumers; see [gameplay review](GAMEPLAY_REVIEW.md) | New binding works, displaced binding follows the chosen policy, prompts update, reset/reload/cancel and controller reconnect behave correctly |
| RBL-06 / P1 | Extend run statistics: results now accurately label transient `score.chain` as Finishing Chain; maximum chain and run comparison remain work | A run statistics owner retains maximum chain, time and best-score comparison across transitions; results/save/retry tests agree |
| RBL-07 / P1 | Add bounded, opt-in raw frame telemetry; current boss diagnostics use smoothed values | Repeatable release-build runs report raw-frame percentiles, maxima, threshold counts and entity peaks without unbounded storage or online collection |
| RBL-08 / P1 | Repair signed environment-grid bounds, explosion-cap overshoot, and mobile-profile ordering; investigate per-projectile environment allocation | Behavioral edge cases pass; comparable fixtures measure the changed path; effect readability survives the cap |
| RBL-09 / P1 | Establish slice asset approval from existing registry work; runtime art mixes backgrounds, cutouts and placeholders | Approve one representative player/enemy/boss set and threat palette first; then validate every asset actually used by the slice |
| RBL-10 / P1 | Make builds and publication traceable; CI ignores asset-only changes and publishing workflows run separately from quality checks | Asset changes run validation; locked builds and matching browser bindings produce identifiable candidates; publishing promotes the tested artifact through an explicit release step |
| RBL-11 / P2 | Investigate narrower Egui features, optional developer gizmos and sprite-first model fallback | Required visuals survive both relevant configurations; measured build/package benefit justifies the change; no missing boss/wingman rendering |
| RBL-12 / expansion | Repair Elder Fleet selection and saved unlocks using its actual nine-mission state | Native selector starts the selected mission, completion persists once, and re-entry after the final mission works; qualify the complete campaign before public exposure |
| RBL-13 / P1 before public release | Prepare the portfolio and community handoff; the existing contributor guide has stale paths and expansion-first priorities | Verified play/download instructions, a 60–90-second gameplay capture, concise engineering case study, current contributor setup and a reproducible playtest/bug-report template |
| RBL-14 / P1 | Finish advertised hull ability effects; speed is now connected, while range, resistance, close-range damage and Afterburner invulnerability assignments lack runtime consumers in the inspected paths | Offered hull abilities have verified activation, actual combat effects, capacitor cost, cooldown and expiry; range/distance rules are explicit and covered by integration tests. Verify instant Salvo/Rocket Barrage activation while the ordinary weapon is cooling down |

Keep state ownership simple: each campaign owns its mission index and
completion; menus display that state; one persistence path records completed
progress. Keep transient combat combo state separate from run statistics.
Consolidate duplicate damage/death paths only with actual weapon, scoring and
boss-completion regressions. Retain the existing ECS structure and save API;
introduce no database, service layer or general-purpose editor for this demo.

Materials flow: editable sources → generated review candidates → per-hull
approval → runtime assets → packaged manifest. Reuse the existing ship
manifest and its migration work rather than adding a competing registry.
The historical archive stays recoverable; new generated previews stay under
ignored build output. Runtime assets total 17.68 MB at the September 7 chapter checkpoint, including the transparent source hulls and orbital title backdrop. The earlier repair baseline measured 13.70 MB. Record subsequent
increases with their player-visible purpose; budget compressed packages only
after measuring actual candidates.

Proposed performance gate: stable 60 FPS on the named reference device.
Initially target p95 raw frame time at or below 16.67 ms and p99 at or below
33.33 ms in steady combat; validate or revise these targets from the first
release-build baseline. Capture three equivalent 60-second windows after
warmup for ordinary waves, dense combat and a boss. Record resolution,
device, effective effects profile, build revision and entity peaks. Track
loading/transition stalls separately. Existing dev-profile CPU microbenchmarks
do not establish this gate.

## First sprint and production cadence

**Sprint 1 outcome: a reproducible, complete candidate and an evidence-based
combat backlog.** Suggested capacity is two focused workweeks; rescope it
after the initial packaged run if progression or save defects are larger.

1. Settle RBL-01 and build RBL-02; capture the current route with isolated saves.
2. Implement RBL-03 and route blockers from RBL-04 before risking real progress.
3. Complete RBL-05 and RBL-06 within the same playable route.
4. Establish RBL-07's rendered baseline and RBL-09's small art approval sample.
5. Prepare RBL-10's candidate/publication separation before the first shared build.
6. Observe initial playtests; rank the next sprint by player impact. Prototype
   the Patrol Commander's first distinct pattern after the route is stable.

Engineering capacity overflow moves telemetry or the boss prototype into
the next sprint. Save integrity, route completion and trustworthy packaging
remain the first gate. An art reviewer can prepare the sample concurrently.

| Responsibility | Accountable role |
| --- | --- |
| Scope, desired combat feel, art acceptance and release decision | Owner/creative lead |
| Implementation, code review, regression checks, profiling and package preparation | Engineering, assisted by Codex |
| Hull preparation, animation/VFX consistency and asset provenance | Artist or designated asset reviewer |
| Physical controls, playability, readability and complete candidate runs | Owner plus external playtesters |

Use short work cycles: one playable objective, small reviewable changes,
appropriate automated checks, one actual play session, and an updated defect
list. Limit active work to one gameplay fix and one independent art/tooling
task per available contributor. A ticket closes only with its evidence;
passing tests alone does not close a visual or feel requirement. This plan
does not create scheduled tasks or send invitations to testers.

Release candidate checklist:

- No open crash, softlock, save-loss or impossible-progression defects in an offered route.
- All offered ships complete the three missions; pause/death/retry/quit/reload and input-device changes pass.
- A clean extracted package works without development paths, cached downloads or a Rust installation.
- Settings, continued progress and best scores survive relaunch; recovery behavior is tested.
- Required asset paths resolve; every used hull has a review/provenance record; threats remain readable under reduced effects.
- Automated checks, asset validation, physical-controller acceptance and rendered timing gates pass for each claimed platform.
- Candidate revision, build inputs, asset manifest, hashes, known limitations and previous working package are recorded.
- Credits, asset provenance and free-project public copy are ready; the owner approves this specific artifact for publishing.
- The portfolio entry links to the playable artifact, source and measured work; community instructions describe how to reproduce a bug or make a bounded contribution.

Browser qualification follows native acceptance: match `wasm-bindgen` tooling
to the lockfile, serve a clean bundle, and test initial load, storage failure,
audio activation, resize/focus and supported input in named browsers. Treat
mobile as a separate device/performance/input qualification, not an automatic
consequence of a successful WASM compile.

## Free portfolio and community release

Owner decision, 2026-09-07: Rebellion will be free and serves as a personal
portfolio and community project. This supersedes the historical proposal for
paid campaign expansions in `PRODUCT_POSITIONING.md`. Later campaigns remain
free; archive availability communicates development status or gameplay progress.

| Deliverable | Purpose and acceptance |
| --- | --- |
| Playable project page | Clear free play/download link for qualified platforms, controls, screenshots, source link, build version and known limitations; add browser play when its qualification passes |
| Gameplay capture | A 60–90-second clip showing movement, distinctive weapons, threat readability and a boss; capture the actual candidate |
| Engineering case study | A concise explanation of the owner's design decisions, architecture, difficult fixes and measured results, linked to code/tests; distinguish asset-size reductions and CPU measurements from rendered performance |
| Community playtest kit | Short three-mission checklist and feedback/bug template with build version, platform, input device, reproduction steps and expected/actual behavior |
| Contributor entry points | Accurate clean-checkout instructions and a few bounded tasks tied to the finishing slice, such as reproducing a bug, reviewing a hull or improving a documented test |

Keep contributor effort focused on a maintainable game and useful feedback.
Use the repository's existing contribution workflow. Measure success through
successful plays, voluntary replays, understandable portfolio evidence and
useful community contributions.

Retain source/permission records and proper credits for EVE and other
third-party assets as part of normal release preparation. The applicable
[CCP content terms](https://support.eveonline.com/hc/en-us/articles/8563917741084-EVE-Online-Content-Creation-Terms-of-Use)
remain linked for that work. This planning update publishes nothing and
contacts no community members.

Source evidence: [repair/verification record](COMPLETION_PLAN.md),
[code and materials review](CODE_REVIEW.md), [gameplay review](GAMEPLAY_REVIEW.md),
[performance review](PERFORMANCE_AUDIT.md), [CI workflow](../.github/workflows/ci.yml),
[release workflow](../.github/workflows/release.yml),
[Pages workflow](../.github/workflows/pages.yml),
[save implementation](../src/core/save.rs),
[CG results screens](../src/games/caldari_gallente/cg_screens.rs),
[historical positioning](PRODUCT_POSITIONING.md) and
[historical slice definition](VERTICAL_SLICE.md).
