# Third-Party Notices

SealyLauncher is licensed under `GPL-3.0-or-later` (see [LICENSE](LICENSE)).
This file records third-party components and reference material.

## Runtime and UI dependencies

| Component | License | Notes |
| --- | --- | --- |
| [GPUI Kit](https://github.com/longbridge/gpui-kit) / `gpui-kit`, `gpui-component`, `gpui-base`, `gpui-kit-assets` | Apache-2.0 | UI framework and component layer |
| [GPUI (Zed)](https://github.com/zed-industries/zed) / `gpui-pre-*` snapshots | Apache-2.0 | GPU-accelerated UI framework |
| [Lucide](https://lucide.dev) icon set (via `gpui-kit-assets`) | ISC | Icons |
| [Inter](https://rsms.me/inter/) | SIL Open Font License 1.1 | Planned Latin UI font |
| [Noto Sans SC](https://fonts.google.com/noto/specimen/Noto+Sans+SC) | SIL Open Font License 1.1 | Planned CJK UI font |

## Runtime-downloaded components (not distributed with the launcher)

- **authlib-injector** — downloaded at runtime from its upstream release metadata; its license and notices are available from the upstream project. It is not bundled in portable packages.
- **Mojang Java runtime** — downloaded at runtime from Mojang's official runtime manifest; it is not bundled in portable packages.
- **Minecraft game files, assets, libraries** — downloaded at runtime from official or mirror sources; no Minecraft game assets are distributed in this repository.

## Behavioral reference

- **HMCL (Hello Minecraft! Launcher)** — GPL-3.0-or-later WITH AdditionRef-HMCL.
  This project references HMCL's public protocols and observed launcher behavior for compatibility.
  No HMCL source code is copied into SealyLauncher. If any HMCL code is ever copied or adapted,
  the affected file and distribution must comply with HMCL's license, including its additional terms.

## Trademarks

Minecraft is a trademark of Mojang Studios. Microsoft, Xbox, and Microsoft accounts are trademarks
of Microsoft Corporation. SealyLauncher is not affiliated with, endorsed by, or sponsored by
Mojang Studios or Microsoft.
