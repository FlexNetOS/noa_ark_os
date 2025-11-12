# NOA Unified Shell – Desktop Adapter

This package bundles the unified shell into a Tauri desktop experience. The configuration wires the shared React dashboard build output into a secure desktop window so operators can run the complete control plane locally.

## Building

```bash
npm install --prefix ../../ui/noa-dashboard
npm run build --prefix ../../ui/noa-dashboard
cargo tauri build --manifest-path ./src-tauri/Cargo.toml
```

The `tauri.conf.json` file specifies the shared build pipeline and ensures the bundle identifier and window chrome match the unified shell branding.
