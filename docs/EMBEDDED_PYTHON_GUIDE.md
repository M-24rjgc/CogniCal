# Embedded Python Runtime Guide for Tauri

## Overview

This guide explains how to embed a Python runtime into your Tauri application, enabling:
- Zero-configuration deployment
- Plugin system with Python
- AI agent ecosystem
- Cross-platform consistency

## Architecture

```
CogniCal.app/
├── CogniCal.exe (or .app on Mac)
├── resources/
│   └── python/
│       ├── python.exe (Windows) / python (Unix)
│       ├── lib/
│       │   └── python3.x/
│       │       └── site-packages/
│       │           ├── kb_mcp_server/
│       │           ├── txtai/
│       │           └── ... (all dependencies)
│       └── Scripts/ (Windows) or bin/ (Unix)
└── plugins/ (user-installable Python plugins)
```

## Implementation Steps

### Step 1: Download Python Embeddable Package

**Windows:**
```powershell
# Download Python 3.11 embeddable (13MB)
$pythonUrl = "https://www.python.org/ftp/python/3.11.9/python-3.11.9-embed-amd64.zip"
Invoke-WebRequest -Uri $pythonUrl -OutFile "python-embed.zip"
Expand-Archive python-embed.zip -DestinationPath "resources/python"
```

**macOS/Linux:**
```bash
# Use python-build-standalone (smaller, optimized builds)
# https://github.com/indygreg/python-build-standalone
wget https://github.com/indygreg/python-build-standalone/releases/download/20240107/cpython-3.11.7+20240107-x86_64-unknown-linux-gnu-install_only.tar.gz
tar -xzf cpython-*.tar.gz -C resources/python
```

### Step 2: Install Dependencies into Embedded Python

```bash
# Use the embedded Python's pip
cd resources/python

# Windows
python.exe -m ensurepip
python.exe -m pip install kb-mcp-server txtai

# Unix
./bin/python3 -m ensurepip
./bin/python3 -m pip install kb-mcp-server txtai
```

### Step 3: Modify Tauri Build Configuration

**tauri.conf.json:**
```json
{
  "bundle": {
    "resources": [
      "resources/python/**/*"
    ],
    "externalBin": []
  }
}
```

### Step 4: Update MCP Client to Use Embedded Python

**src-tauri/src/services/mcp_client.rs:**

```rust
use std::path::PathBuf;
use tauri::Manager;

impl McpClient {
    /// Get the path to the embedded Python executable
    fn get_embedded_python_path() -> AppResult<PathBuf> {
        // Get the resource directory
        let resource_dir = tauri::api::path::resource_dir(
            &tauri::generate_context!().config().package,
            &tauri::Env::default()
        ).ok_or_else(|| AppError::other("Failed to get resource directory"))?;
        
        let python_path = if cfg!(target_os = "windows") {
            resource_dir.join("python").join("python.exe")
        } else {
            resource_dir.join("python").join("bin").join("python3")
        };
        
        if !python_path.exists() {
            return Err(AppError::other(format!(
                "Embedded Python not found at: {}",
                python_path.display()
            )));
        }
        
        Ok(python_path)
    }
    
    /// Spawn MCP server using embedded Python
    pub fn spawn(kb_path: &Path) -> AppResult<Self> {
        info!(
            target: "mcp_client",
            kb_path = %kb_path.display(),
            "Spawning MCP server with embedded Python"
        );
        
        // Try embedded Python first
        let python_path = Self::get_embedded_python_path()?;
        
        info!(
            target: "mcp_client",
            python_path = %python_path.display(),
            "Using embedded Python"
        );
        
        let mut child = Command::new(&python_path)
            .args(&["-m", "kb_mcp_server", "--kb-path", kb_path.to_str().unwrap()])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|e| {
                error!(
                    target: "mcp_client",
                    error = %e,
                    "Failed to spawn MCP server with embedded Python"
                );
                AppError::other(format!("Failed to spawn MCP server: {}", e))
            })?;
        
        // ... rest of the implementation
    }
}
```

### Step 5: Build Script to Automate Packaging

**build.rs** (in src-tauri/):

```rust
use std::process::Command;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    
    // Only run during release builds
    if std::env::var("PROFILE").unwrap() == "release" {
        setup_embedded_python();
    }
}

fn setup_embedded_python() {
    let python_dir = Path::new("resources/python");
    
    if !python_dir.exists() {
        println!("cargo:warning=Setting up embedded Python...");
        
        // Download and extract Python
        if cfg!(target_os = "windows") {
            setup_windows_python();
        } else if cfg!(target_os = "macos") {
            setup_macos_python();
        } else {
            setup_linux_python();
        }
        
        // Install dependencies
        install_python_dependencies();
    }
}

fn setup_windows_python() {
    // Download Python embeddable
    Command::new("powershell")
        .args(&[
            "-Command",
            "Invoke-WebRequest -Uri 'https://www.python.org/ftp/python/3.11.9/python-3.11.9-embed-amd64.zip' -OutFile 'python-embed.zip'; Expand-Archive python-embed.zip -DestinationPath 'resources/python'"
        ])
        .status()
        .expect("Failed to download Python");
}

fn install_python_dependencies() {
    let python_exe = if cfg!(target_os = "windows") {
        "resources/python/python.exe"
    } else {
        "resources/python/bin/python3"
    };
    
    // Install pip
    Command::new(python_exe)
        .args(&["-m", "ensurepip"])
        .status()
        .expect("Failed to install pip");
    
    // Install dependencies
    Command::new(python_exe)
        .args(&["-m", "pip", "install", "kb-mcp-server", "txtai"])
        .status()
        .expect("Failed to install Python dependencies");
}
```

## Size Optimization

### Minimal Python Build

**Only include necessary modules:**

```python
# create_minimal_python.py
import os
import shutil
from pathlib import Path

def create_minimal_python(source_dir, dest_dir):
    """Create a minimal Python distribution with only required modules"""
    
    # Core modules to keep
    keep_modules = {
        'kb_mcp_server',
        'txtai',
        'numpy',
        'sqlite3',
        'json',
        'asyncio',
        # Add other required modules
    }
    
    # Copy Python executable
    shutil.copy(f"{source_dir}/python.exe", dest_dir)
    
    # Copy only required site-packages
    site_packages_src = f"{source_dir}/Lib/site-packages"
    site_packages_dst = f"{dest_dir}/Lib/site-packages"
    
    for module in keep_modules:
        src = f"{site_packages_src}/{module}"
        if os.path.exists(src):
            shutil.copytree(src, f"{site_packages_dst}/{module}")
```

**Expected sizes:**
- Full Python: ~50MB
- Minimal Python: ~20MB
- With dependencies: ~80MB total

## Plugin System Architecture

### Plugin Structure

```python
# plugins/my_agent_plugin/
# ├── __init__.py
# ├── manifest.json
# └── agent.py

# manifest.json
{
  "name": "My Custom Agent",
  "version": "1.0.0",
  "author": "Your Name",
  "description": "A custom AI agent",
  "entry_point": "agent.py",
  "permissions": ["filesystem", "network"],
  "dependencies": ["requests", "beautifulsoup4"]
}

# agent.py
class CustomAgent:
    def __init__(self, config):
        self.config = config
    
    async def process(self, input_data):
        # Your agent logic
        return result
```

### Plugin Manager (Rust)

```rust
pub struct PluginManager {
    python_runtime: Arc<PythonRuntime>,
    plugins: HashMap<String, Plugin>,
}

impl PluginManager {
    pub async fn load_plugin(&mut self, plugin_path: &Path) -> AppResult<()> {
        // Read manifest
        let manifest = self.read_manifest(plugin_path)?;
        
        // Install dependencies
        self.install_dependencies(&manifest.dependencies).await?;
        
        // Load Python module
        let plugin = self.python_runtime.load_module(plugin_path)?;
        
        self.plugins.insert(manifest.name, plugin);
        Ok(())
    }
    
    pub async fn execute_plugin(&self, name: &str, input: Value) -> AppResult<Value> {
        let plugin = self.plugins.get(name)
            .ok_or_else(|| AppError::other("Plugin not found"))?;
        
        plugin.execute(input).await
    }
}
```

## Benefits for Your Ecosystem

### 1. Zero-Configuration Deployment
- Users install one app, everything works
- No "install Python first" instructions
- Consistent experience across all platforms

### 2. Plugin Marketplace
```
CogniCal Plugin Store
├── AI Agents
│   ├── Code Assistant Agent
│   ├── Research Agent
│   └── Writing Assistant
├── Integrations
│   ├── Notion Sync
│   ├── Google Calendar
│   └── Slack Bot
└── Tools
    ├── PDF Parser
    ├── Web Scraper
    └── Data Analyzer
```

### 3. Sandboxed Execution
- Plugins run in isolated Python processes
- Permission system controls access
- Can't crash the main app

### 4. Hot Reload
- Update plugins without restarting app
- Develop plugins with live reload
- Easy debugging

## Alternative: PyO3 (Pure Rust Integration)

For even better integration, consider **PyO3**:

```rust
use pyo3::prelude::*;

#[pyfunction]
fn call_from_rust(input: String) -> PyResult<String> {
    // Call Python code from Rust
    Python::with_gil(|py| {
        let module = py.import("kb_mcp_server")?;
        let result: String = module.call_method1("process", (input,))?.extract()?;
        Ok(result)
    })
}
```

**Benefits:**
- No subprocess overhead
- Direct memory sharing
- Better performance
- Smaller binary

**Tradeoffs:**
- More complex build process
- Harder to sandbox
- Platform-specific compilation

## Recommended Approach

**Phase 1** (Current): External Python dependency
**Phase 2** (Next release): Embedded Python runtime
**Phase 3** (Future): PyO3 integration + Plugin marketplace

## Implementation Checklist

- [ ] Download Python embeddable packages for all platforms
- [ ] Create build script to package Python
- [ ] Update MCP client to use embedded Python
- [ ] Test on clean machines (no Python installed)
- [ ] Optimize Python distribution size
- [ ] Add fallback to system Python if embedded fails
- [ ] Document plugin development
- [ ] Create plugin SDK
- [ ] Build plugin marketplace UI

## Resources

- [Python Embeddable Package](https://www.python.org/downloads/windows/)
- [python-build-standalone](https://github.com/indygreg/python-build-standalone)
- [PyO3 Documentation](https://pyo3.rs/)
- [Tauri Resource Bundling](https://tauri.app/v1/guides/building/resources)

---

**This approach will make CogniCal a true platform for AI agents and productivity tools!** 🚀
