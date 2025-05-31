# AGENT.md - AGDK Winit WGPU Project

## Build Commands
- `cargo check` - Type check the project
- `cargo build` - Build for desktop (requires "desktop" feature)
- `cargo build --features desktop` - Build desktop binary
- `cargo clippy` - Run linter
- `cargo fmt` - Format code
- `cargo ndk -t arm64-v8a -o app/src/main/jniLibs/ build` - Build for Android
- `./gradlew build` - Build Android APK
- `./gradlew installDebug` - Install on Android device

## Testing
No formal test suite configured. Run builds to verify code.

## Code Style
- Use `snake_case` for variables, functions, modules
- Use `PascalCase` for types, structs, enums
- Prefer explicit types for struct fields
- Use `pub` for public APIs, default private
- Error handling: Use `anyhow::Result` for functions that can fail
- Imports: Group std, external crates, then local modules
- No trailing commas in single-line arrays/structs
- Use `#[rustfmt::skip]` for matrices that should preserve formatting
