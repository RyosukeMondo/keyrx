# PowerShell Pester Tests

## Requirements

The diagnostic_scripts_test.ps1 requires **Pester 5.x** or later.

### Current System

The current system has Pester 3.4.0 installed, which uses different syntax.

### Installing Pester 5.x

```powershell
# Install Pester 5.x
Install-Module -Name Pester -Force -SkipPublisherCheck -MinimumVersion 5.0

# Verify installation
Get-Module -Name Pester -ListAvailable

# Import the module
Import-Module Pester -MinimumVersion 5.0
```

### Running Tests

```powershell
# Run all tests
Invoke-Pester -Path tests/diagnostic_scripts_test.ps1

# Run with detailed output
Invoke-Pester -Path tests/diagnostic_scripts_test.ps1 -Output Detailed

# Run specific test
Invoke-Pester -Path tests/diagnostic_scripts_test.ps1 -TestName "Version Check*"
```

## Syntax Differences

### Pester 3.x (Current)
```powershell
$value | Should Be "expected"
Test-Path $file | Should Be $true
```

### Pester 5.x (Required)
```powershell
$value | Should -Be "expected"
$file | Should -Exist
$content | Should -Match "pattern"
```

## Test Coverage

The test suite covers:
- Installer health checks
- Installation diagnostics
- Version validation
- Error handling
- PowerShell script behavior

## CI/CD Integration

For CI/CD environments, ensure Pester 5.x is installed before running tests:

```yaml
- name: Install Pester 5.x
  run: Install-Module -Name Pester -Force -SkipPublisherCheck -MinimumVersion 5.0

- name: Run PowerShell tests
  run: Invoke-Pester -Path tests/diagnostic_scripts_test.ps1 -CI
```
