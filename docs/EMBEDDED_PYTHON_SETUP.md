# Embedded Python Setup

## Quick Start

### For Development

1. **Setup embedded Python** (one-time):
   ```powershell
   .\setup_python.ps1
   ```

2. **Run the app**:
   ```powershell
   pnpm tauri dev
   ```

The app will automatically use the embedded Python for memory service.

### For Production Build

1. **Setup embedded Python** (if not already done):
   ```powershell
   .\setup_python.ps1
   ```

2. **Build the app**:
   ```powershell
   pnpm tauri build
   ```

The embedded Python will be automatically included in the installer.

## What Gets Installed

The setup script will:
1. Download Python 3.11.9 embeddable package (~13MB)
2. Install pip
3. Install memory service and dependencies (~60MB)
4. Configure Python paths

**Total size**: ~80MB

## Directory Structure

```
CogniCal/
├── resources/
│   └── python/
│       ├── python.exe              # Python executable
│       ├── python311.dll           # Python runtime
│       ├── python311._pth          # Path configuration
│       ├── Lib/
│       │   └── site-packages/
│       │       ├── memory_service/  # Memory service
│       │       ├── txtai/          # AI library
│       │       └── ...             # Dependencies
│       └── Scripts/
│           └── pip.exe             # Package manager
```

## How It Works

### Development Mode

1. App looks for embedded Python in `resources/python/`
2. If found, uses it to run memory service
3. If not found, falls back to system Python

### Production Mode

1. Tauri bundles `resources/python/**/*` into the app
2. App extracts Python to installation directory
3. Always uses embedded Python (no system dependency)

## Fallback Behavior

The app tries Python in this order:

1. **Embedded Python** (highest priority)
   - `resources/python/python.exe`
   
2. **System Python**
   - `python -m memory_service`
   - `python3 -m memory_service`
   
3. **Direct command**
   - `memory-service`

This ensures the app works even if embedded Python setup fails.

## Troubleshooting

### "Failed to download Python"

**Solution**: Check your internet connection and try again.

### "Failed to install pip"

**Solution**: 
1. Delete `resources/python` folder
2. Run `.\setup_python.ps1` again

### "memory_service module not found"

**Solution**:
1. Check if `resources/python/Lib/site-packages/memory_service` exists
2. If not, run:
   ```powershell
   .\resources\python\python.exe -m pip install memory-service
   ```

### Embedded Python not found in production

**Solution**:
1. Make sure you ran `.\setup_python.ps1` before building
2. Check that `tauri.conf.json` includes:
   ```json
   "resources": ["resources/python/**/*"]
   ```

## Manual Setup

If the automatic script fails, you can set up manually:

```powershell
# 1. Download Python embeddable
Invoke-WebRequest -Uri "https://www.python.org/ftp/python/3.11.9/python-3.11.9-embed-amd64.zip" -OutFile "python.zip"

# 2. Extract
Expand-Archive python.zip -DestinationPath "resources/python"

# 3. Enable site-packages
# Edit resources/python/python311._pth
# Uncomment: import site
# Add line: Lib/site-packages

# 4. Install pip
Invoke-WebRequest -Uri "https://bootstrap.pypa.io/get-pip.py" -OutFile "get-pip.py"
.\resources\python\python.exe get-pip.py

# 5. Install packages
.\resources\python\python.exe -m pip install memory-service txtai
```

## Size Optimization

To reduce the embedded Python size:

### Remove unnecessary files

```powershell
# Remove test files
Remove-Item -Recurse resources/python/Lib/site-packages/*/tests

# Remove documentation
Remove-Item -Recurse resources/python/Lib/site-packages/*/docs

# Remove examples
Remove-Item -Recurse resources/python/Lib/site-packages/*/examples
```

### Use minimal dependencies

Edit `scripts/setup_embedded_python.ps1` to install only required packages.

## CI/CD Integration

### GitHub Actions

```yaml
- name: Setup Embedded Python
  run: |
    pwsh -File setup_python.ps1
  
- name: Build Tauri App
  run: |
    pnpm tauri build
```

### GitLab CI

```yaml
build:
  script:
    - pwsh -File setup_python.ps1
    - pnpm tauri build
```

## Future Improvements

- [ ] Cross-platform support (macOS, Linux)
- [ ] Automatic updates for Python packages
- [ ] Plugin system using embedded Python
- [ ] Python version management
- [ ] Minimal Python builds (< 30MB)

## Benefits

✅ **Zero Configuration**: Users don't need to install Python
✅ **Consistent**: Same Python version on all machines
✅ **Isolated**: No conflicts with system Python
✅ **Portable**: App works anywhere
✅ **Plugin Ready**: Foundation for plugin ecosystem

---

**Questions?** Check the [Embedded Python Guide](./EMBEDDED_PYTHON_GUIDE.md) for more details.
