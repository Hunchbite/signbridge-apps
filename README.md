# SignBridge Apps

**Hunchbite product layer** on top of the [`signbridge`](https://github.com/Hunchbite/signbridge) framework:

| App | Role |
|---|---|
| `signbridge` CLI | Scripts / humans |
| `signbridge-host` | Chrome/Firefox native-messaging host |
| `extension/` | Browser extension (page ↔ host) |
| `packaging/` | Native messaging manifests (Mac / Windows installers later) |

## Talking to core from any language

The framework repo owns the language-agnostic surfaces (`signbridge-rpc`, C ABI, Python/TS clients). See https://github.com/Hunchbite/signbridge/blob/main/docs/PROTOCOL.md.

This repo adds Hunchbite product UX on top (CLI, browser host, extension).

## Setup

Clone next to the framework (path dependency):

```text
Desktop/Code/
├── signbridge/          # framework
└── signbridge-apps/     # this repo
```

```bash
cd signbridge-apps
cargo build -p signbridge-cli -p signbridge-host
cargo run -p signbridge-cli -- ping
cargo run -p signbridge-cli -- doctor   # India / Spain / EU PKCS#11 modules on this machine
cargo run -p signbridge-cli -- list
```

Web eID / AutoFirma adapters live in the framework (`signbridge-rpc --protocol webeid|afirma`). See https://github.com/Hunchbite/signbridge/blob/main/docs/ADAPTERS.md.

To depend on GitHub instead of the local path, edit the workspace `Cargo.toml`:

```toml
signbridge = { git = "https://github.com/Hunchbite/signbridge", tag = "v0.1.0" }
```

## Extension

1. Build `signbridge-host` and install somewhere stable.
2. Point `packaging/macos/com.hunchbite.signbridge.json` `path` at the binary; set your extension id in `allowed_origins`.
3. Install the host manifest under Chrome’s `NativeMessagingHosts/`.
4. Load `extension/` unpacked in `chrome://extensions`.

## Licence

MIT OR Apache-2.0 · [Hunchbite Technologies Private Limited](https://hunchbite.com)
