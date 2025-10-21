# 🚀 Embedded Python Quick Start

## TL;DR

```powershell
# 1. Setup embedded Python (one-time, ~2 minutes)
.\setup_python.ps1

# 2. Run the app
pnpm tauri dev

# 3. Build for production
pnpm tauri build
```

That's it! Memory features will work out of the box.

## What Just Happened?

### Before (Current State)
- ❌ Users need to install Python manually
- ❌ Users need to install memory service dependencies
- ❌ Different Python versions cause issues
- ❌ Complex setup instructions

### After (With Embedded Python)
- ✅ Python bundled with the app
- ✅ Zero configuration for users
- ✅ Consistent across all machines
- ✅ Just download and run

## Step-by-Step Guide

### 1. Setup Embedded Python

Run the setup script:

```powershell
.\setup_python.ps1
```

This will:
- Download Python 3.11.9 embeddable (~13MB)
- Install pip
- Install memory service dependencies
- Configure everything automatically

**Time**: ~2 minutes  
**Size**: ~80MB  
**Frequency**: One-time (or when updating Python packages)

### 2. Verify Setup

Check that Python was installed:

```powershell
.\resources\python\python.exe --version
# Should output: Python 3.11.9

.\resources\python\python.exe -c "import sys; print('OK')"
# Should output: OK
```

### 3. Development

Run the app normally:

```powershell
pnpm tauri dev
```

The app will automatically find and use the embedded Python.

### 4. Production Build

Build the app:

```powershell
pnpm tauri build
```

The embedded Python will be automatically included in:
- Windows: `src-tauri/target/release/bundle/nsis/CogniCal_1.0.1014_x64-setup.exe`
- The installer will extract Python to the installation directory

## How It Works

### Priority Order

The app tries to find Python in this order:

1. **Embedded Python** (highest priority)
   ```
   resources/python/python.exe
   ```

2. **System Python** (fallback)
   ```
   python -m memory_service
   python3 -m memory_service
   ```

3. **Direct command** (last resort)
   ```
   memory-service
   ```

### Development vs Production

**Development** (`pnpm tauri dev`):
- Looks for `resources/python/` in project root
- Falls back to system Python if not found

**Production** (`pnpm tauri build`):
- Bundles `resources/python/` into the installer
- Extracts to installation directory
- Always uses embedded Python

## File Structure

```
CogniCal/
├── setup_python.ps1              # Quick setup script
├── scripts/
│   └── setup_embedded_python.ps1 # Detailed setup script
├── resources/
│   └── python/                   # Embedded Python (gitignored)
│       ├── python.exe
│       ├── python311.dll
│       ├── Lib/
│       │   └── site-packages/
│       │       ├── memory_service/
│       │       └── txtai/
│       └── Scripts/
│           └── pip.exe
└── src-tauri/
    ├── tauri.conf.json          # Configured to bundle Python
    └── src/
        └── services/
            └── memory_client.rs    # Updated to use embedded Python
```

## Troubleshooting

### "glob pattern resources/python/**/* path not found"

This is **normal** if you haven't run the setup script yet.

**Solution**: Run `.\setup_python.ps1`

### Setup script fails

**Solution**: Run the detailed script with verbose output:
```powershell
.\scripts\setup_embedded_python.ps1 -Verbose
```

### Python not found in production build

**Checklist**:
- [ ] Did you run `.\setup_python.ps1` before building?
- [ ] Does `resources/python/python.exe` exist?
- [ ] Is `tauri.conf.json` configured correctly?

### Memory features still not working

**Debug steps**:
1. Check logs for "Successfully spawned memory service"
2. Verify Python path in logs
3. Test Python manually:
   ```powershell
   .\resources\python\python.exe -m memory_service --kb-path ./test_kb
   ```

## CI/CD Integration

### GitHub Actions

```yaml
name: Build

on: [push]

jobs:
  build:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Node
        uses: actions/setup-node@v3
        with:
          node-version: 18
      
      - name: Install dependencies
        run: pnpm install
      
      - name: Setup Embedded Python
        run: .\setup_python.ps1
      
      - name: Build Tauri App
        run: pnpm tauri build
      
      - name: Upload Installer
        uses: actions/upload-artifact@v3
        with:
          name: CogniCal-Installer
          path: src-tauri/target/release/bundle/nsis/*.exe
```

## Benefits

### For Users
- ✅ Download and run - no setup needed
- ✅ No Python installation required
- ✅ No dependency conflicts
- ✅ Consistent experience

### For Developers
- ✅ Reproducible builds
- ✅ Version control
- ✅ Easy testing
- ✅ No environment issues

### For the Product
- ✅ Lower barrier to entry
- ✅ Better user experience
- ✅ Foundation for plugin system
- ✅ Professional deployment

## Next Steps

### Phase 1: Basic Embedded Python ✅
- [x] Setup script
- [x] Tauri configuration
- [x] Memory client updates
- [x] Documentation

### Phase 2: Optimization
- [ ] Reduce Python size (< 50MB)
- [ ] Faster setup script
- [ ] Cross-platform support (macOS, Linux)
- [ ] Automatic updates

### Phase 3: Plugin System
- [ ] Plugin API
- [ ] Plugin manager UI
- [ ] Plugin marketplace
- [ ] Hot reload

## FAQ

**Q: Do I need to commit `resources/python/` to git?**  
A: No, it's gitignored. Each developer runs the setup script.

**Q: How big is the final installer?**  
A: ~100MB (app ~20MB + Python ~80MB)

**Q: Can users still use system Python?**  
A: Yes, the app falls back to system Python if embedded Python is not found.

**Q: What if I want to update Python packages?**  
A: Run `.\resources\python\python.exe -m pip install --upgrade memory-service`

**Q: Does this work on macOS/Linux?**  
A: Not yet, but the architecture supports it. Coming soon!

---

**Ready to try it?** Run `.\setup_python.ps1` now! 🚀
