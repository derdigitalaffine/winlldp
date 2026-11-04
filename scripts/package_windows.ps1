# Package tested Windows x64 release artifacts from an MSVC build.
# Usage: cargo test --locked --target x86_64-pc-windows-msvc
#        cargo build --release --locked --target x86_64-pc-windows-msvc --bins
#        pwsh -File scripts/package_windows.ps1
$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$version = "1.0.0"
$target = "x86_64-pc-windows-msvc"
$releaseDir = Join-Path "target" "$target/release"
$buildDir = "build"
$gui = Join-Path $releaseDir "winlldp.exe"
$cli = Join-Path $releaseDir "winlldp-cli.exe"

if (-not $IsWindows) { throw "This package must be built and tested on Windows." }
foreach ($file in @($gui, $cli, "installer/winlldp.iss")) {
    if (-not (Test-Path -LiteralPath $file)) { throw "Missing required input: $file" }
}

# A GUI-subsystem executable does not reliably return synchronously from a normal shell.
$smokeReport = Join-Path $env:TEMP "winlldp-self-test-$PID.txt"
try {
    $process = Start-Process -FilePath (Resolve-Path $gui).Path -ArgumentList @("--self-test", "--report", $smokeReport) -Wait -PassThru
    if ($process.ExitCode -ne 0) { throw "WinLLDP self-test exited with $($process.ExitCode)" }
    if ((-not (Test-Path -LiteralPath $smokeReport)) -or
        ((Get-Content -LiteralPath $smokeReport -Raw) -notmatch "WinLLDP Windows self-test OK")) {
        throw "GUI executable Windows self-test did not create a valid report"
    }
} finally {
    Remove-Item -LiteralPath $smokeReport -Force -ErrorAction SilentlyContinue
}

# The CLI must be a real runnable console binary.
& $cli --help
if ($LASTEXITCODE -ne 0) { throw "CLI --help returned $LASTEXITCODE" }

if (Test-Path -LiteralPath $buildDir) { Remove-Item -LiteralPath $buildDir -Recurse -Force }
New-Item -ItemType Directory -Path $buildDir | Out-Null
Copy-Item -LiteralPath $gui -Destination "$buildDir/WinLLDP-$version-Windows-x64.exe"
Copy-Item -LiteralPath $cli -Destination "$buildDir/winlldp-cli-$version-Windows-x64.exe"
Copy-Item -LiteralPath "LICENSE" -Destination "$buildDir/LICENSE.txt"
Copy-Item -LiteralPath "THIRD_PARTY_NOTICES.md" -Destination "$buildDir/THIRD_PARTY_NOTICES.md"
Copy-Item -LiteralPath "Cargo.lock" -Destination "$buildDir/Cargo.lock"

$isccPaths = @(
    "${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe",
    "$env:ProgramFiles\Inno Setup 6\ISCC.exe"
)
$iscc = $isccPaths | Where-Object { Test-Path -LiteralPath $_ } | Select-Object -First 1
if (-not $iscc) {
    $command = Get-Command ISCC.exe -ErrorAction SilentlyContinue
    if ($command) { $iscc = $command.Source }
}
if (-not $iscc) { throw "Inno Setup 6 ISCC.exe was not found. Install Inno Setup 6 first." }
& $iscc "installer/winlldp.iss"
if ($LASTEXITCODE -ne 0) { throw "Inno Setup returned $LASTEXITCODE" }

$setup = "$buildDir/WinLLDP-$version-Windows-x64-Setup.exe"
if (-not (Test-Path -LiteralPath $setup)) { throw "Installer was not created: $setup" }
$sourceSha = (& git rev-parse HEAD).Trim()
$rustVersion = (& rustc --version).Trim()
@(
    "WinLLDP $version (Windows 10/11 x64)"
    "Source commit: $sourceSha"
    "Rust: $rustVersion"
    "Rust target: $target"
    "Build: cargo build --locked --release --target $target --bins"
    "Tests: cargo test --locked --target $target"
    "Smoke: WinLLDP.exe --self-test --report FILE, and winlldp-cli.exe --help (passed)"
    "Installer: Inno Setup 6, per-user installation, includes uninstall entry"
    "Release binaries: unsigned; verify SHA256SUMS.txt before use"
    "Npcap: external optional driver, NOT bundled; required for live LLDP access"
    "Hardware/network capture and installation/uninstallation on actual user PCs: not validated in CI"
) | Set-Content -LiteralPath "$buildDir/BUILD_INFO.txt" -Encoding utf8

$executables = @(Get-ChildItem -LiteralPath $buildDir -Filter "*.exe" -File | Sort-Object Name)
if ($executables.Count -ne 3) { throw "Expected GUI, CLI and setup EXEs; found $($executables.Count)" }
$shaLines = foreach ($file in $executables) {
    $hash = (Get-FileHash -LiteralPath $file.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
    "$hash  $($file.Name)"
}
$shaLines | Set-Content -LiteralPath "$buildDir/SHA256SUMS.txt" -Encoding utf8
Write-Host "Windows binaries, installer and SHA256 checksums prepared in $buildDir"
