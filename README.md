# fs-watch

📁 A lightweight file system watcher for logging directory changes in real-time.

**⚠️ Windows-only:** this project uses the native `ReadDirectoryChangesW` API via `notify`, and currently only supports Windows.

---

## 🚀 Features

- Watches for file and directory events: `Create`, `Modify`, `Remove`
- Outputs structured JSON events to `stdout`
- Supports glob-based path filtering (opt-in or opt-out)
- Fast startup with initial recursive scan

---

## 🛠️ Build

```bash
cargo build --release
