param(
    [string]$OutputDir = "dist/windows-playtest",
    [string]$Python = "python"
)

# Run on Windows with the MSVC toolchain. Fetch dependencies before packaging.
Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
if ([Environment]::OSVersion.Platform -ne [PlatformID]::Win32NT) {
    throw "Build this package on Windows with the MSVC toolchain."
}

function Invoke-Checked {
    param([string]$Program, [string[]]$Arguments)
    & $Program @Arguments
    if ($LASTEXITCODE -ne 0) {
        throw "$Program failed with exit code $LASTEXITCODE"
    }
}

function Write-Utf8 {
    param([string]$Path, [string]$Text)
    [IO.File]::WriteAllText($Path, $Text, [Text.UTF8Encoding]::new($false))
}

$RootDir = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$Target = "x86_64-pc-windows-msvc"
$Name = "Rebellion-windows-x86_64"
$PreviousDirectory = (Get-Location).Path
$PreviousRustFlags = [Environment]::GetEnvironmentVariable("RUSTFLAGS", "Process")
$PreviousEncodedFlags = [Environment]::GetEnvironmentVariable("CARGO_ENCODED_RUSTFLAGS", "Process")
$StagingDir = $null
try {
    Set-Location -LiteralPath $RootDir
    foreach ($Program in @("cargo", "rustc", "git", $Python)) {
        Get-Command $Program -ErrorAction Stop | Out-Null
    }
    Invoke-Checked $Python @("--version")
    if (-not [IO.Path]::IsPathRooted($OutputDir)) {
        $OutputDir = Join-Path $RootDir $OutputDir
    }
    $OutputDir = [IO.Path]::GetFullPath($OutputDir)
    [IO.Directory]::CreateDirectory($OutputDir) | Out-Null
    $Destination = Join-Path $OutputDir $Name
    $Archive = "$Destination.zip"
    foreach ($Path in @($Destination, $Archive, "$Archive.sha256")) {
        if (Test-Path -LiteralPath $Path) {
            throw "Destination exists; choose a fresh output directory: $Path"
        }
    }

    # Bundle the CRT into this executable. Target-specific compilation avoids
    # applying the static CRT setting to host proc macros and build scripts.
    $StaticCrt = "-C target-feature=+crt-static"
    $env:RUSTFLAGS = "$PreviousRustFlags $StaticCrt".Trim()
    if ($null -ne $PreviousEncodedFlags) {
        $Separator = [char]31
        $Flags = @()
        if ($PreviousEncodedFlags.Length -gt 0) {
            $Flags += $PreviousEncodedFlags
        }
        $Flags += "-C"
        $Flags += "target-feature=+crt-static"
        $env:CARGO_ENCODED_RUSTFLAGS = $Flags -join $Separator
    }
    $TargetDir = Join-Path $RootDir "target"
    Invoke-Checked "cargo" @("build", "--offline", "--locked", "--release", "--bin", "rebellion", "--target", $Target, "--target-dir", $TargetDir)
    $Binary = Join-Path $TargetDir "$Target/release/rebellion.exe"
    $Bytes = [IO.File]::ReadAllBytes($Binary)
    if ($Bytes.Length -lt 64 -or $Bytes[0] -ne 0x4d -or $Bytes[1] -ne 0x5a) {
        throw "Expected a Windows PE executable: $Binary"
    }
    $PeOffset = [BitConverter]::ToInt32($Bytes, 0x3c)
    if ($PeOffset -lt 64 -or $PeOffset -gt ($Bytes.Length - 26) -or
        [BitConverter]::ToUInt32($Bytes, $PeOffset) -ne 0x00004550 -or
        [BitConverter]::ToUInt16($Bytes, $PeOffset + 4) -ne 0x8664 -or
        [BitConverter]::ToUInt16($Bytes, $PeOffset + 24) -ne 0x020b) {
        throw "Expected a 64-bit x86_64 Windows PE executable: $Binary"
    }

    $StagingDir = Join-Path $OutputDir (".rebellion-package-" + [Guid]::NewGuid().ToString("N"))
    $Package = Join-Path $StagingDir $Name
    [IO.Directory]::CreateDirectory($Package) | Out-Null
    Copy-Item -LiteralPath $Binary -Destination (Join-Path $Package "rebellion.exe")
    Copy-Item -LiteralPath (Join-Path $RootDir "assets") -Destination (Join-Path $Package "assets") -Recurse
    Copy-Item -LiteralPath (Join-Path $RootDir "LICENSE") -Destination (Join-Path $Package "LICENSE")
    Copy-Item -LiteralPath (Join-Path $RootDir "docs/WINDOWS_PLAYTEST.md") -Destination (Join-Path $Package "README.md")

    $ManifestTool = Join-Path $RootDir "scripts/playtest-manifest.py"
    $Manifest = Join-Path $Package "PLAYTEST-MANIFEST.json"
    Invoke-Checked $Python @($ManifestTool, "write", "--target", $Target, "--output", $Manifest)
    Invoke-Checked $Python @($ManifestTool, "verify", "--manifest", $Manifest, "--assets", (Join-Path $Package "assets"))
    $Source = (Invoke-Checked "git" @("rev-parse", "HEAD") | Out-String).Trim()
    $Changes = (Invoke-Checked "git" @("status", "--porcelain", "--untracked-files=normal") | Out-String).Trim()
    $TreeState = if ($Changes.Length -gt 0) { "modified (includes local changes)" } else { "clean" }
    $Compiler = (Invoke-Checked "rustc" @("--version") | Out-String).Trim()
    $BuildInfo = @(
        "Rebellion Windows playtest"
        "Target: $Target"
        "Built: $([DateTime]::UtcNow.ToString('yyyy-MM-ddTHH:mm:ssZ'))"
        "Source: $Source"
        "Working tree: $TreeState"
        "Runtime: static MSVC CRT; Windows system libraries remain required"
        "Compiler: $Compiler"
        "Content identity: PLAYTEST-MANIFEST.json"
    ) -join "`n"
    Write-Utf8 (Join-Path $Package "BUILD-INFO.txt") ($BuildInfo + "`n")
    $Checksums = Get-ChildItem -LiteralPath $Package -File -Recurse | Sort-Object FullName | ForEach-Object {
        $Relative = $_.FullName.Substring($Package.Length + 1).Replace('\', '/')
        $Digest = (Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
        "$Digest  $Relative"
    }
    Write-Utf8 (Join-Path $Package "SHA256SUMS") (($Checksums -join "`n") + "`n")
    $StagedArchive = Join-Path $StagingDir "$Name.zip"
    Add-Type -AssemblyName System.IO.Compression.FileSystem
    [IO.Compression.ZipFile]::CreateFromDirectory($Package, $StagedArchive, [IO.Compression.CompressionLevel]::Optimal, $true)
    $ArchiveHash = (Get-FileHash -LiteralPath $StagedArchive -Algorithm SHA256).Hash.ToLowerInvariant()
    Write-Utf8 "$StagedArchive.sha256" "$ArchiveHash  $Name.zip`n"
    # These move APIs fail if a competing process creates a destination.
    [IO.Directory]::Move($Package, $Destination)
    [IO.File]::Move($StagedArchive, $Archive)
    [IO.File]::Move("$StagedArchive.sha256", "$Archive.sha256")
    Write-Output "Packaged game: $Destination"
    Write-Output "Transfer archive: $Archive"
    Write-Output "Run: $Destination\rebellion.exe"
}
finally {
    [Environment]::SetEnvironmentVariable("RUSTFLAGS", $PreviousRustFlags, "Process")
    [Environment]::SetEnvironmentVariable("CARGO_ENCODED_RUSTFLAGS", $PreviousEncodedFlags, "Process")
    Set-Location -LiteralPath $PreviousDirectory
    if ($null -ne $StagingDir -and (Test-Path -LiteralPath $StagingDir)) {
        Remove-Item -LiteralPath $StagingDir -Recurse -Force
    }
}
