# Ship identity and presentation review

Updated 2026-09-07. The initial review found 38 name/ID conflicts among 74 registry entries. The current registry has 80 hulls and **zero name/ID conflicts** against the dated CCP ESI snapshot. This is an identity check, not final artwork approval.

79 runtime PNGs now come from the owner's `AreteDriver/eve-ship-sprites` repository, pinned at commit `992bab5e19548744f3038e15bae599c93a1a1342`. Each downloaded blob was checked against its Git SHA-1. The import uses the original transparent renders unchanged; the game applies orientation from the manifest. [Source paths and blob hashes](source-import.json).

All imported images have alpha and four fully transparent corners. All 22 selectable hull definitions in the two chapter rosters have matching manifest names and factions. Five invalid or obsolete entries (11373, 11547, 11566, 11568, 35685) were moved to the ignored `build/ship-review/quarantine` folder after correcting references, removing them from the app bundle.

The Gila entry is now type 17715. Its old stylized placeholder remains outside the two exposed chapter rosters and needs authentic replacement before that mode is qualified. Several source silhouettes have ambiguous bow/stern landmarks; contact-sheet rotation review is provisional. Every manifest status remains `pending_review` until final per-hull visual qualification and hardpoint work.

The runtime loads every manifest hull and reads its rotation from that same manifest, eliminating separate preload and orientation lists. Enemy banking, bosses, player previews, wingmen and exhaust preserve gameplay-facing alignment.

## Current inventory

| Type | Hull | Source-facing correction | PNG SHA-256 |
| --- | --- | --- | --- |
| 583 | Condor | 180 degrees | `ccdb88c44468881079588cd0341bd0229aec42e6b8f0780eeb00591eb8176f78` |
| 585 | Slasher | 0 degrees | `7414439548955a90febff61ed24264dc04089df6dd96951c03b3b7266329df39` |
| 586 | Probe | 180 degrees | `2937e2ea1e1ecb13a94cced5a9a201071b0f28c24a3cfc0e7e2decb003371402` |
| 587 | Rifter | 0 degrees | `97f03f41454e581b8d082d23ce8b62f0a86dd22580aee4b836d026e40f6578e3` |
| 589 | Executioner | 180 degrees | `dcf1ea13db555577d3c83cfdb1b0b4ca898123858d6fd2d8942470185bf03d02` |
| 591 | Tormentor | 180 degrees | `af8432e8f80de4c7bd92204eb9af39a872c01180e715f3eb6a4ab858dca75cab` |
| 592 | Navitas | 180 degrees | `006a6649ed2b0b5454f85cf6dffc64f83194e7de6a1a6633fdf81e3ab907dc26` |
| 593 | Tristan | 0 degrees | `ad6d96478b95e0fe71d55d5934835c6b8fa0f11daf1b367ebe90d780db18aac7` |
| 594 | Incursus | 0 degrees | `dcb3319a2ec6be0551c00165c6c7ad120f515ddd59d1e3fab836e6330c254b9d` |
| 597 | Punisher | 180 degrees | `5059e21276f1b507dcda54558f34b909e4a36bdf0754106b8854b6c9332a5cdd` |
| 598 | Breacher | 0 degrees | `b801fb53f6fb5e30ad24e8d60c32ecfdf911e9915184e319ac842ccfb7cfd6e7` |
| 602 | Kestrel | 90 degrees | `6446a1eb17f97e0c4643b3a23e79ad78c936486e935512b490e4d77e7e27cecb` |
| 603 | Merlin | -90 degrees | `236bce85a1aa625af5e1fa51c0bc0eac73dc5cf8fd0a8449449a96fcd56b13f1` |
| 605 | Heron | 0 degrees | `569bfd4ac31d0b97b111257735cf48e381c683b8bd5d5ed67324844d8ecc0489` |
| 608 | Atron | 90 degrees | `a4712753f72a1f4498bd36e1cee2840ea09505f8f8ad44d0a1163d8660cf7ac6` |
| 621 | Caracal | 180 degrees | `6da4ab13a82f059fca41ca0f046200e288d9de0347a5d8d8a4e25fed68ba5656` |
| 622 | Stabber | 0 degrees | `780c43862c4113a0f0ece3a5301b83fcf2721cb1b8a78b755187e7956a4c3476` |
| 623 | Moa | 180 degrees | `f610d1d32d49286ff680e87dae58d4e607c94cf21f184b75bad20c08a6014922` |
| 624 | Maller | 180 degrees | `0ec4ae0349108ccc98a5804cee30a5443a980dbf60995844f391a07b2e9fbc18` |
| 625 | Augoror | 180 degrees | `976cd8c93f10249c7e80c5ce4c9c01b7bf5d77957a8169fbbb8e412089415597` |
| 626 | Vexor | 180 degrees | `52947cd3f1ac21b6d598c4ac2dad065db78c71267856f3f12c1b7a2cbd8f318c` |
| 630 | Bellicose | 180 degrees | `52e34ecb01cc7a240fe416573701ac8f1e68e902bb8f9d7db55fec4229e8e110` |
| 638 | Raven | 180 degrees | `d146e85082d42b2c2bcfd940435469d1b2bd6e5e2f357ce38dc12e077d942268` |
| 639 | Tempest | 0 degrees | `51cc4560d357f80af9463af491e05e7abb766c71d80aa6679ee4e2d718068155` |
| 640 | Scorpion | 180 degrees | `56948e69ff8135d87f32bbba413de6ba8e987ad7e63b5f46e3e23e7c011e20f1` |
| 641 | Megathron | 180 degrees | `9b04c7abeb8eec229ec439a6db321c97e371e33b3ae2e77e55c351f7d25e90d0` |
| 642 | Apocalypse | 180 degrees | `f0e39befb4c8e33db491478bf1597255ec7ccbe1e6d5911b1c08206c426f9c62` |
| 643 | Armageddon | 180 degrees | `8a5aff938f9aeee090e5e514bd7fa769a167d3815f68d476f4cc5e50482934dc` |
| 645 | Dominix | 0 degrees | `e9e144f0ed031e5b2f8e7626920921a3f9b3fa875d47c6a86c0444219983da39` |
| 1944 | Bestower | 180 degrees | `2bc22e7770e318bf8b259b3dacdbac89e3ae1df126ffeccf32481643fa728545` |
| 2006 | Omen | 180 degrees | `c8be14d1caeda244c14eaaf93f569185d6c8971bbfa3d76641ae1e1f91e613ed` |
| 3764 | Leviathan | 0 degrees | `2aff041c5588c757b9bc808989e67fbf8c716252d53409ef591f0fbc23dfe983` |
| 11184 | Crusader | 180 degrees | `f68915fc763e6512b5d2f24f17725787e9ad24bee9da7607c5a18bd14fad201d` |
| 11186 | Malediction | 180 degrees | `9bb658dff1d186ec789f454094e6307abbd8109c8afec9a880256ceac30dbd16` |
| 11371 | Wolf | 180 degrees | `fbe9f921f2384e15ed8ef0b149b8208891c4e8d840588c8331ebc36ce8559e84` |
| 11377 | Nemesis | -90 degrees | `b55db54226d3e4b20fd54bfdc7f34ff1d2b77f1cc54f1867872ccc39cb5d8fd1` |
| 11379 | Hawk | -90 degrees | `c66dd685ec574efb232e8e2f51e7268dadd0f91a26889a289006af2ebe7b3849` |
| 11381 | Harpy | -90 degrees | `f32f4b7de5b751ec7020d9e367aba1197c3d3ea99a31e7b7d95a77a61b6d76e7` |
| 11387 | Hyena | 90 degrees | `7bcc21199cdd059dedb492e5cd5ee42424a8dce0288f58bd430653aa9a2a6355` |
| 11393 | Retribution | 180 degrees | `8744bbfa0c821540dc03c9a96995c95654283e4fe44a3a9e22aae9f6af2dc23f` |
| 11400 | Jaguar | 180 degrees | `8512a7e3c17af153195acfeb641c82428a3112b8fbdcbf177cf58a58753476e5` |
| 11567 | Avatar | 0 degrees | `c7d0f96c040f37469cfe6e334733e13029f62bb5cc9c16dd7f715095b69f308a` |
| 11993 | Cerberus | 180 degrees | `264dc9e6c3429f82fc355b8eb24bbfb213b4ba46b690f2feab8318770917e0e0` |
| 12019 | Sacrilege | 180 degrees | `87a5dea18f4789f0a472020d4d45ce15b55e34f256ede8fcf6cdd69a6cc80a48` |
| 12042 | Ishkur | 0 degrees | `4490322200d0519ab2b4def07e5ecece5f60e7c8a35bc3bb3afa3406c7d78c4b` |
| 12044 | Enyo | 0 degrees | `8e8b4df581a9890e25291ef2fb5529dec4727c579a76d7aa4c58f11976565730` |
| 16227 | Ferox | 0 degrees | `6d5af4e63f28b03df94b9e554b136c67735e6d5358bd366e606414a7a2222e76` |
| 16236 | Coercer | 180 degrees | `5d6e3f28c206fa8538d902e3777e31a391fb347e6fd1e2859bbc9c67c44cd009` |
| 16238 | Cormorant | 0 degrees | `0d5a97ca9db845995e68d877161193a054ee26e495bbbe1b57ce834ddd0636ac` |
| 16240 | Catalyst | 0 degrees | `2bf8aac26c6d4e14c15b7713e00d2fb92abbe5f2b9386d88e4b124e9123e269a` |
| 16242 | Thrasher | 180 degrees | `ddaab2d5be546dc5376d1e4c5a016d778cf0db354ce3313d1379264ef1b9fda0` |
| 17715 | Gila | 0 degrees | `57639a75463d54d71d1d897699eb53794b4687d5d6621507a68d22bf82334422` |
| 20185 | Charon | 0 degrees | `f57e2e3abf8c59dce91f53566c31dc845573d6a3f6260d3adf207c8805d1a41a` |
| 23757 | Archon | 0 degrees | `8c1381001eef2a70a9ef02967e13158fb090d334797de52fc81551b6f9848773` |
| 23911 | Thanatos | 0 degrees | `15e3018e9804fce3f9f8a2eb0e8470984d11fea3b4d1668addc131c8ee32a4d4` |
| 23915 | Chimera | 0 degrees | `c8747aa04d11dc67e94b1f242cddd1941e8930d49618923acd458a0e3b88114d` |
| 24483 | Nidhoggur | 0 degrees | `12a11baec9c6039a1907a59e1bb6a2461ae1bd91a5b30cc3c2930fd2d0759636` |
| 24688 | Rokh | 0 degrees | `18d31b7ac008e2bf75faa8b93a245bc67eeb830a2a1d5240dfad9f4d872d9339` |
| 24690 | Hyperion | 0 degrees | `a5a3132a9013fa3acba7a02219bee8a6072988314a50a473795726ce45ce2b6f` |
| 24692 | Abaddon | 180 degrees | `96f6d4311750a9c432f584eb771252dd469d45334f14ae9c3c904a5651422344` |
| 24694 | Maelstrom | 0 degrees | `e6fd75dfab8f89c86b0afa6415f80f2755d5cd6a373a78f7ef937489fd00dd67` |
| 24696 | Harbinger | 180 degrees | `c13d3ad859167da23c720e27923c8888ee27ceb23ad0b61285a0d662ec0ebabf` |
| 24698 | Drake | 0 degrees | `c374ee109eecd6b6860e04fd2e1437f36747eb68ec18e0a3f67bc237f8ec57ff` |
| 24700 | Myrmidon | 90 degrees | `672b81300c2a324806040c4904821392e7e3a89de4aa317330029a5b1d2cb1f0` |
| 24702 | Hurricane | 0 degrees | `99befbe36b00150a18a7c1f45746b3d230ef0c2d0e6be6791076517faea06d87` |
| 33818 | Orthrus | 180 degrees | `81ee10b1ef64fad1a8b620f772045107b8d77a657a690ccfc02d82ae7405579f` |
| 34317 | Confessor | 180 degrees | `25a50436a63c05ce6a4450b7dd644cf476ff1c59a7fcb97af1c0f9c6ba3b3683` |
| 34828 | Jackdaw | 0 degrees | `82592a3e69fefe2d393ecf3fd5f7a19168b10ea75fa19796ace247cbf4a314f3` |
| 35683 | Hecate | 0 degrees | `0de3ac7316e0b5026678441a05a0d947c54197fd0a6b7223831b133306cba65c` |
| 47269 | Damavik | 180 degrees | `bde1597f83476704c293cd42197a00381c23a10c39d06ce8e5e7aadb4359cb81` |
| 47270 | Vedmak | 180 degrees | `3ff097374c446df7dca6a2b89ccc1d0507c21831f5e0ae85700584a440f47310` |
| 47271 | Leshak | 0 degrees | `26d16f8268343008cf9ec8cd776d15ae7ff2c2f4beaf61c8b093e1d42318ee9c` |
| 49710 | Kikimora | 90 degrees | `363100609fd7ccee32f1f5f970a2c1e60cd1cc7af89c1996e29dd74d74d60245` |
| 49711 | Drekavac | 180 degrees | `0696c2462efbd03548082abe0f66f8973034b7e66adf02b6f64adf826e2b111e` |
| 52250 | Nergal | 180 degrees | `fc0f98fa1791da05301a5ec4c5e710668324ed98f2332642cd9607f0f681e28e` |
| 52252 | Ikitursa | 180 degrees | `5cc16e94c9846b03735122a39d7d427ea8d6c9e152e09b0cabb1270316f8cc81` |
| 52254 | Draugur | 0 degrees | `0dd8157760b448ad3c3710b824bfed815ef5facbabe428d5c41627353208d1cc` |
| 54731 | Skybreaker | 90 degrees | `297cde82b04158706bfe461e3cba2c1345d2116244b2754900da6f3530f212bd` |
| 54732 | Stormbringer | -90 degrees | `d484d6e5a67c8c0be8c3cb6fa510b390317ded96c9537a3a121b683ddecb509f` |
| 54733 | Thunderchild | -90 degrees | `d291e6bcf5bc6fb2fbff6dcb8134d8db5648905043c085f0bc0d796eb2ed8f94` |
