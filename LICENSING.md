# Licensing

Copyright (C) 2026 BlockVerse contributors

BlockVerse is free and open-source software. Anyone may use, study, modify and share it.
The only condition: if you distribute BlockVerse or a modified version, you must share the
complete source code under the same license and keep the copyright notices.

| Part | License |
|---|---|
| Server core (`crates/bv-server`, `crates/bv-server`, `native/host`, other crates) | **GPL-3.0-or-later** + [Plugin Exception](PLUGIN-EXCEPTION.md) (full text: [LICENSE](LICENSE)) |
| Plugin ABI header and SDKs (`native/include`, `crates/bv-plugin-abi`) | **Apache-2.0** ([LICENSE-APACHE-2.0](LICENSE-APACHE-2.0)) |
| Example plugins (`crates/bv-example-plugin`, `native/plugins`) | **Apache-2.0** |
| Default skins (`crates/bv-skins/assets`) | **CC BY 4.0** (see `crates/bv-skins/assets/LICENSE.md`) |
| Third-party code (see [NOTICE](NOTICE)) | their own licenses |

## Why this model

- **GPL-3.0-or-later for the core** keeps every distributed fork open and keeps the original authorship visible.
- **Plugin Exception + Apache-2.0 SDK** let plugin authors choose any license, including closed source, as long as
  plugins talk to the server only through the plugin ABI.
- **Apache-2.0 for the ABI/SDK/examples** so nobody is blocked from writing plugins.
- **CC BY 4.0 for skins**: free to use and remix, with credit.

## Not covered by these licenses

- The names "BlockVerse" and "BlockWare" and the logo ([TRADEMARKS.md](TRADEMARKS.md)).
- Mojang's protocol documentation and any data or code taken directly from it (all rights reserved by Mojang,
  subject to the Minecraft EULA). Never commit those files.

This file is a plain-language summary, not legal advice.
