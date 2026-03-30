# Windows Installer Comparison

This document compares the different Windows installer approaches for KeyRx.

## 📊 Quick Comparison

| Feature | IExpress | PowerShell Script | WiX (MSI) |
|---------|----------|-------------------|-----------|
| **External Tools Required** | None | None | Yes (WiX Toolset) |
| **Build Complexity** | Medium | Low | High |
| **User Experience** | Professional | Command-line | Professional |
| **File Format** | .exe | .ps1 | .msi |
| **File Size** | ~5 MB | ~7 MB | ~5 MB |
| **Windows Integration** | Good | Basic | Excellent |
| **Silent Install** | Yes | Yes | Yes |
| **Uninstaller** | Script-based | Script-based | Built-in |
| **Code Signing** | Yes | Yes | Yes |
| **Distribution** | Easy | Easy | Easy |
| **Customization** | Limited | Easy | Extensive |
| **Maintenance** | Low | Low | High |

## 🎯 Detailed Comparison

### IExpress (Recommended for Most Users)

**What it is:** Built-in Windows tool for creating self-extracting installers.

**Advantages:**
- ✅ Built into Windows (no installation required)
- ✅ Creates professional .exe installer
- ✅ Can display license and custom prompts
- ✅ Supports silent installation
- ✅ Familiar to users (just run the .exe)
- ✅ Small file size
- ✅ Easy to script and automate

**Disadvantages:**
- ❌ Limited customization options
- ❌ No built-in uninstaller (must provide script)
- ❌ Less control over installation UI
- ❌ Windows-only (can't build on Linux)

**Best for:**
- Quick distribution to end users
- Simple installation requirements
- When no external tools are allowed
- Automated builds in CI/CD

**Build Command:**
```powershell
.\scripts\installer\build-installer.ps1
```

**Output:** `keyrx-installer-v1.0.0.exe`

---

### PowerShell Script (Simplest Approach)

**What it is:** Single PowerShell script with all files embedded as Base64.

**Advantages:**
- ✅ Simplest approach (one script does everything)
- ✅ Easy to inspect and modify
- ✅ No build tools required
- ✅ Can be distributed as-is
- ✅ Full PowerShell flexibility
- ✅ Transparent to users

**Disadvantages:**
- ❌ Larger file size (~33% overhead from Base64)
- ❌ Users must run PowerShell
- ❌ Less professional appearance
- ❌ Execution policy may block
- ❌ Some users may distrust scripts

**Best for:**
- Internal distribution
- Power users comfortable with PowerShell
- When transparency is important
- Quick testing and iteration

**Build Command:**
```powershell
.\scripts\installer\create-simple-installer.ps1
```

**Output:** `keyrx-installer-v1.0.0.ps1`

---

### WiX (MSI) - Not Implemented Yet

**What it is:** Professional Windows Installer using WiX Toolset.

**Advantages:**
- ✅ Professional enterprise-grade installer
- ✅ Full Windows Installer integration
- ✅ Built-in uninstaller
- ✅ Upgrade/repair support
- ✅ Group Policy deployment
- ✅ Extensive customization
- ✅ Rollback support

**Disadvantages:**
- ❌ Requires WiX Toolset installation
- ❌ Steep learning curve
- ❌ Complex XML configuration
- ❌ Harder to maintain
- ❌ Slower build times

**Best for:**
- Enterprise deployment
- Complex installation requirements
- When maximum Windows integration is needed
- Professional commercial software

**Build Command:**
```bash
make msi  # Uses scripts/windows/build_msi.bat
```

**Output:** `keyrx-v1.0.0.msi`

**Note:** Currently available but requires WiX Toolset installation.

## 🔍 Feature-by-Feature Breakdown

### Installation Experience

| Feature | IExpress | PowerShell | WiX (MSI) |
|---------|----------|------------|-----------|
| Double-click to install | ✅ | ❌ | ✅ |
| Custom prompts | ✅ | ✅ | ✅ |
| License display | ✅ | ✅ | ✅ |
| Progress bar | ❌ | ❌ | ✅ |
| Custom branding | ❌ | ❌ | ✅ |
| User selectable options | ❌ | ❌ | ✅ |

### Technical Capabilities

| Feature | IExpress | PowerShell | WiX (MSI) |
|---------|----------|------------|-----------|
| Add to PATH | ✅ | ✅ | ✅ |
| Create shortcuts | ✅ | ✅ | ✅ |
| Register file types | ❌ | ✅ | ✅ |
| Windows Services | ❌ | ✅ | ✅ |
| Registry modifications | ❌ | ✅ | ✅ |
| Custom actions | ❌ | ✅ | ✅ |
| Rollback support | ❌ | ❌ | ✅ |

### Distribution & Deployment

| Feature | IExpress | PowerShell | WiX (MSI) |
|---------|----------|------------|-----------|
| Silent installation | ✅ | ✅ | ✅ |
| Command-line options | ❌ | ✅ | ✅ |
| Group Policy deployment | ❌ | ❌ | ✅ |
| Windows Update | ❌ | ❌ | ✅ |
| SCCM deployment | ❌ | ❌ | ✅ |
| Code signing | ✅ | ✅ | ✅ |

### Maintenance

| Feature | IExpress | PowerShell | WiX (MSI) |
|---------|----------|------------|-----------|
| Easy to modify | ✅ | ✅ | ❌ |
| Easy to debug | ✅ | ✅ | ❌ |
| Versioning | Manual | Manual | Built-in |
| Upgrade support | ❌ | ❌ | ✅ |
| Repair support | ❌ | ❌ | ✅ |

## 💡 Recommendations

### For Individual Users/Open Source Projects

**Recommended:** IExpress

**Why:**
- No external tools required
- Professional appearance
- Easy distribution
- Good enough for most users

**Alternative:** PowerShell script for power users

---

### For Enterprise/Commercial Software

**Recommended:** WiX (MSI)

**Why:**
- Professional enterprise requirements
- Full Windows integration
- Upgrade/repair support
- Group Policy deployment

**Alternative:** IExpress for quick internal tools

---

### For Internal Tools/Utilities

**Recommended:** PowerShell Script

**Why:**
- Simplest to create and modify
- Easy to customize
- Transparent source code
- IT teams comfortable with PowerShell

**Alternative:** IExpress for non-technical users

---

### For CI/CD Pipelines

**Recommended:** IExpress

**Why:**
- Built into Windows (no setup)
- Scriptable and automatable
- Fast build times
- Consistent results

**Alternative:** PowerShell for quick iterations

## 📈 Performance Comparison

### Build Time

| Installer | Build Time | Notes |
|-----------|------------|-------|
| PowerShell | ~5 seconds | Just Base64 encoding |
| IExpress | ~30 seconds | Extraction + compression |
| WiX (MSI) | ~2 minutes | Complex compilation |

### File Size

| Installer | Size | Compression |
|-----------|------|-------------|
| PowerShell | ~7 MB | None (Base64 overhead) |
| IExpress | ~5 MB | CAB compression |
| WiX (MSI) | ~5 MB | MSI compression |

### Installation Time

| Installer | Time | Notes |
|-----------|------|-------|
| PowerShell | ~10 seconds | Direct file writes |
| IExpress | ~15 seconds | Extraction + script |
| WiX (MSI) | ~20 seconds | Full MSI process |

## 🎓 Learning Curve

### PowerShell Script

**Difficulty:** Easy

**Learning required:**
- Basic PowerShell syntax
- File operations
- Registry editing (optional)

**Time to proficiency:** 1-2 hours

---

### IExpress

**Difficulty:** Medium

**Learning required:**
- IExpress GUI/SED format
- PowerShell for installation script
- Windows installation concepts

**Time to proficiency:** 2-4 hours

---

### WiX (MSI)

**Difficulty:** Hard

**Learning required:**
- WiX XML syntax
- Windows Installer concepts
- Component/feature modeling
- Custom actions
- Upgrade logic

**Time to proficiency:** 1-2 weeks

## 🔒 Security Considerations

### PowerShell Script

**Risks:**
- Execution policy may block
- Users may distrust scripts
- Easy to inspect = easy to modify

**Mitigations:**
- Code signing
- Checksums
- Clear documentation

---

### IExpress

**Risks:**
- Windows SmartScreen warnings (unsigned)
- Less transparent than script

**Mitigations:**
- Code signing certificate
- Checksums
- Official distribution channels

---

### WiX (MSI)

**Risks:**
- Complex = more attack surface
- Custom actions can be dangerous

**Mitigations:**
- Code signing (required)
- Security audits
- Minimal custom actions

## 📝 Summary

### Choose IExpress if:
- ✅ You want a professional installer
- ✅ No external tools allowed
- ✅ Simple installation requirements
- ✅ Distributing to end users

### Choose PowerShell if:
- ✅ You want the simplest approach
- ✅ Transparency is important
- ✅ Distributing to power users
- ✅ Frequent customization needed

### Choose WiX (MSI) if:
- ✅ Enterprise deployment required
- ✅ Complex installation logic needed
- ✅ Full Windows integration required
- ✅ Upgrade/repair support needed

## 🚀 Getting Started

### IExpress Quick Start
```powershell
.\scripts\installer\build-installer.ps1
# Output: keyrx-installer-v1.0.0.exe
```

### PowerShell Quick Start
```powershell
.\scripts\installer\create-simple-installer.ps1
# Output: keyrx-installer-v1.0.0.ps1
```

### Build Both
```powershell
.\scripts\installer\quick-build.ps1 -Type Both
```

## 📚 Additional Resources

- [IExpress Guide](./INSTALLER_GUIDE.md)
- [Build Scripts](./README.md)
- [WiX Documentation](https://wixtoolset.org/docs/)

---

**For KeyRx, we recommend starting with IExpress for distribution and PowerShell for development/testing.**
