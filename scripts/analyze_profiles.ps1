# Analyze Profile Files - No Admin Required
# Identifies issues with profile files that might cause activation failures

Write-Host "========================================" -ForegroundColor Cyan
Write-Host " Profile Analysis Tool" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

$configDir = "C:\Users\$env:USERNAME\AppData\Roaming\keyrx\profiles"

if (-not (Test-Path $configDir)) {
    Write-Host "ERROR: Config directory not found: $configDir" -ForegroundColor Red
    exit 1
}

Write-Host "Config directory: $configDir" -ForegroundColor Gray
Write-Host ""

# Analyze all profiles
Get-ChildItem -Path $configDir -Filter "*.rhai" | ForEach-Object {
    $profileName = $_.BaseName
    $rhaiFile = $_.FullName
    $krxFile = Join-Path $configDir "$profileName.krx"

    Write-Host "=== $profileName ===" -ForegroundColor Yellow

    # File sizes
    $rhaiSize = (Get-Item $rhaiFile).Length
    Write-Host "  Source (.rhai): $rhaiSize bytes" -ForegroundColor $(if ($rhaiSize -gt 10000) { "Yellow" } else { "White" })

    if ($rhaiSize -gt 10000) {
        Write-Host "    WARNING: Large file may cause timeout!" -ForegroundColor Red
    }

    if (Test-Path $krxFile) {
        $krxSize = (Get-Item $krxFile).Length
        $krxTime = (Get-Item $krxFile).LastWriteTime
        Write-Host "  Compiled (.krx): $krxSize bytes (updated: $($krxTime.ToString('yyyy/MM/dd HH:mm:ss')))"

        # Check if krx is stale
        $rhaiTime = (Get-Item $rhaiFile).LastWriteTime
        if ($rhaiTime -gt $krxTime) {
            Write-Host "    WARNING: .krx is older than .rhai - needs recompilation!" -ForegroundColor Yellow
        }
    } else {
        Write-Host "  Compiled (.krx): NOT FOUND" -ForegroundColor Red
        Write-Host "    ERROR: Profile has not been compiled!" -ForegroundColor Red
    }

    # Content analysis
    $content = Get-Content $rhaiFile -Raw

    # Count remappings
    $remapCount = ([regex]::Matches($content, "remap_key")).Count
    Write-Host "  Remappings: $remapCount" -ForegroundColor $(if ($remapCount -gt 100) { "Yellow" } else { "White" })

    # Count layers
    $layerCount = ([regex]::Matches($content, "add_layer")).Count
    Write-Host "  Layers: $layerCount"

    # Count macros
    $macroCount = ([regex]::Matches($content, "define_macro")).Count
    Write-Host "  Macros: $macroCount"

    # Check for syntax issues
    if ($content -match "\b(KEY_\w+)\s*,\s*\)") {
        Write-Host "  WARNING: Possible syntax error (missing argument)" -ForegroundColor Yellow
    }

    # Complexity score
    $complexity = $remapCount + ($layerCount * 10) + ($macroCount * 5)
    Write-Host "  Complexity score: $complexity" -ForegroundColor $(
        if ($complexity -gt 500) { "Red" }
        elseif ($complexity -gt 200) { "Yellow" }
        else { "Green" }
    )

    if ($complexity -gt 500) {
        Write-Host "    WARNING: Very complex profile may cause issues!" -ForegroundColor Red
    }

    # Show first few lines
    Write-Host "  First 10 lines:"
    $content -split "`n" | Select-Object -First 10 | ForEach-Object {
        Write-Host "    $_" -ForegroundColor Gray
    }

    Write-Host ""
}

# Check which profile is active
Write-Host "=== Active Profile ===" -ForegroundColor Yellow
try {
    $profiles = Invoke-RestMethod -Uri http://localhost:9867/api/profiles -TimeoutSec 5 -ErrorAction Stop
    $activeProfile = $profiles.profiles | Where-Object { $_.active -eq $true }

    if ($activeProfile) {
        Write-Host "  Active: $($activeProfile.name)" -ForegroundColor Green
        Write-Host "  Status: $($activeProfile.status)"
    } else {
        Write-Host "  None" -ForegroundColor Yellow
    }
} catch {
    Write-Host "  ERROR: Cannot connect to API" -ForegroundColor Red
    Write-Host "  Make sure daemon is running" -ForegroundColor Gray
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "=== Analysis Complete ===" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

Write-Host "Recommendations:" -ForegroundColor Yellow
Write-Host "1. Files >10KB may cause activation timeout" -ForegroundColor White
Write-Host "2. Complexity score >500 may cause performance issues" -ForegroundColor White
Write-Host "3. Stale .krx files need recompilation" -ForegroundColor White
Write-Host ""
