# NOA Unified Shell – Mobile Adapter

The mobile adapter exposes the unified control plane through a React Native shell. The `app.json` configuration defines the Expo runtime metadata and points the application to the unified API gateway so on-device actions map to the same workflows and chat interfaces as the web experience.

## Running with Expo

```bash
cd apps/mobile-shell
npx expo start --dev-client
```

Before launching, ensure the unified FastAPI gateway is reachable from your device or emulator so chat commands and workflow triggers operate seamlessly.
