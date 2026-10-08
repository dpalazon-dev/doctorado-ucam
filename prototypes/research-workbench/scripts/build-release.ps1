[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$script:ReleaseVersion = '0.0.1'
$script:ReleaseTarget = 'x86_64-pc-windows-msvc'
$script:InstallName = "Research-Workbench_$($script:ReleaseVersion)_x64-setup.exe"

function Get-UniqueInstaller {
    param([string[]]$Paths)

    if ($Paths.Count -ne 1) {
        throw "Expected exactly one NSIS setup from this build; found $($Paths.Count)."
    }
    return $Paths[0]
}

function Get-PeMachine {
    param([Parameter(Mandatory)][string]$Path)

    $stream = [IO.File]::OpenRead($Path)
    $reader = $null
    try {
        $reader = [IO.BinaryReader]::new($stream)
        if ($reader.ReadUInt16() -ne 0x5a4d) { throw "Not a PE executable: $Path" }
        $stream.Position = 0x3c
        $peOffset = $reader.ReadInt32()
        if ($peOffset -lt 0 -or $peOffset -gt ($stream.Length - 6)) { throw "Invalid PE header offset: $Path" }
        $stream.Position = $peOffset
        if ($reader.ReadUInt32() -ne 0x00004550) { throw "Invalid PE signature: $Path" }
        $machine = $reader.ReadUInt16()
        return $machine
    } finally {
        if ($reader) { $reader.Dispose() } else { $stream.Dispose() }
    }
}

function Get-PeArchitecture {
    param([Parameter(Mandatory)][int]$Machine)
    switch ($Machine) {
        0x014c { return 'i386' }
        0x8664 { return 'x64' }
        0xaa64 { return 'ARM64' }
        default { return 'unknown' }
    }
}

function Assert-PeX64 {
    param([Parameter(Mandatory)][string]$Path)
    $machine = Get-PeMachine -Path $Path
    if ($machine -ne 0x8664) { throw ('Expected x64 PE machine 0x8664, found 0x{0:x4}: {1}' -f $machine, $Path) }
    return $machine
}

function Assert-NsisSetupMachine {
    param([Parameter(Mandatory)][string]$Path)
    $machine = Get-PeMachine -Path $Path
    if ($machine -notin @(0x014c, 0x8664)) {
        throw ('Unsupported NSIS setup PE machine 0x{0:x4}: {1}' -f $machine, $Path)
    }
    return $machine
}

function Assert-RustcHost {
    param([Parameter(Mandatory)][string]$Details)
    $hostLines = [regex]::Matches($Details, '(?m)^host:\s*(?<target>[^\r\n]+)\r?$')
    if ($hostLines.Count -ne 1 -or $hostLines[0].Groups['target'].Value.Trim() -ne $script:ReleaseTarget) {
        throw "Release host must be $($script:ReleaseTarget). rustc reports: $Details"
    }
}

function Format-PeMachine {
    param([Parameter(Mandatory)][int]$Machine)
    return ('{0} (0x{1:x4})' -f (Get-PeArchitecture -Machine $Machine), $Machine)
}

function Assert-ReleaseVersionContract {
    param(
        [Parameter(Mandatory)][string]$PackageVersion,
        [Parameter(Mandatory)][string]$TauriVersion,
        [Parameter(Mandatory)][string]$CargoVersion
    )

    $versions = @($PackageVersion, $TauriVersion, $CargoVersion)
    $mismatches = @($versions | Where-Object { $_ -ne $script:ReleaseVersion })
    if ($mismatches.Count -gt 0) {
        throw "Pilot release requires package, Tauri, and Cargo versions to equal $($script:ReleaseVersion). Found: $($versions -join ', ')."
    }
}

function Get-Sha256 {
    param([Parameter(Mandatory)][string]$Path)
    return (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
}

function Get-LockHashes {
    param([Parameter(Mandatory)][string]$Root)
    $lockPaths = @('package-lock.json', 'src-tauri/Cargo.lock')
    $hashes = [ordered]@{}
    foreach ($relativePath in $lockPaths) {
        $path = Join-Path $Root $relativePath
        if (-not (Test-Path -LiteralPath $path -PathType Leaf)) { throw "Required lockfile is missing: $relativePath" }
        $hashes[$relativePath] = Get-Sha256 -Path $path
    }
    return $hashes
}

function Get-GitValue {
    param([Parameter(Mandatory)][string]$Root, [Parameter(Mandatory)][string[]]$Arguments)
    $value = & git -C $Root @Arguments
    if ($LASTEXITCODE -ne 0) { throw "git $($Arguments -join ' ') failed with exit code $LASTEXITCODE." }
    return (($value | Out-String).Trim())
}

function Assert-CleanSourceTree {
    param([Parameter(Mandatory)][string]$Root)
    $status = Get-GitValue -Root $Root -Arguments @('status', '--porcelain=v1', '--untracked-files=normal')
    if ($status) { throw "Source tree must be clean before and after release build. git status: $status" }
}

function Get-ToolOutput {
    param([Parameter(Mandatory)][string]$Command, [string[]]$Arguments = @())
    $output = & $Command @Arguments 2>&1
    if ($LASTEXITCODE -ne 0) { throw "$Command $($Arguments -join ' ') failed with exit code $LASTEXITCODE." }
    return (($output | Out-String).Trim())
}

function Invoke-ReleaseBuild {
    $root = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
    $package = Get-Content -LiteralPath (Join-Path $root 'package.json') -Raw | ConvertFrom-Json
    $tauri = Get-Content -LiteralPath (Join-Path $root 'src-tauri/tauri.conf.json') -Raw | ConvertFrom-Json
    $cargoText = Get-Content -LiteralPath (Join-Path $root 'src-tauri/Cargo.toml') -Raw
    if ($cargoText -notmatch '(?m)^version\s*=\s*"(?<version>[^\"]+)"\s*$') {
        throw 'Could not read the src-tauri package version from Cargo.toml.'
    }
    Assert-ReleaseVersionContract -PackageVersion $package.version -TauriVersion $tauri.version -CargoVersion $Matches.version

    if (-not $tauri.bundle.active -or @($tauri.bundle.targets) -notcontains 'nsis') {
        throw 'Tauri configuration must enable the NSIS bundle for the pilot.'
    }
    if ($tauri.bundle.windows.nsis.installMode -ne 'currentUser' -or $tauri.bundle.windows.webviewInstallMode.type -ne 'offlineInstaller') {
        throw 'Pilot config must keep currentUser installation and offlineInstaller WebView2 mode.'
    }

    Assert-CleanSourceTree -Root $root
    $sourceSha = Get-GitValue -Root $root -Arguments @('rev-parse', 'HEAD')
    $lockHashesBefore = Get-LockHashes -Root $root
    $releaseDirectory = Join-Path $root "dist-release/$($script:ReleaseVersion)"
    if (Test-Path -LiteralPath $releaseDirectory) {
        throw "Refusing to overwrite an existing frozen release directory: $releaseDirectory"
    }
    $null = & git -C $root check-ignore -q dist-release/0.0.1
    if ($LASTEXITCODE -ne 0) { throw 'dist-release/0.0.1 must be ignored by Git before packaging.' }

    . (Join-Path $PSScriptRoot 'development-env.ps1')
    $rustcDetails = Get-ToolOutput -Command 'rustc.exe' -Arguments @('-vV')
    Assert-RustcHost -Details $rustcDetails

    $commonGitDirectory = Get-GitValue -Root $root -Arguments @('rev-parse', '--path-format=absolute', '--git-common-dir')
    $commonRepositoryRoot = Split-Path $commonGitDirectory -Parent
    $runId = [guid]::NewGuid().ToString('N')
    $cargoTargetRoot = Join-Path $commonRepositoryRoot "work/release-build/$($script:ReleaseVersion)-$runId"
    if (Test-Path -LiteralPath $cargoTargetRoot) { throw "Unique build directory unexpectedly exists: $cargoTargetRoot" }
    New-Item -ItemType Directory -Path $cargoTargetRoot -Force | Out-Null
    $env:CARGO_TARGET_DIR = $cargoTargetRoot

    $command = @('npm.cmd', 'run', 'tauri:build', '--', '--target', $script:ReleaseTarget, '--bundles', 'nsis', '--ci')
    Push-Location $root
    try {
        & npm.cmd run tauri:build -- --target $script:ReleaseTarget --bundles nsis --ci
        if ($LASTEXITCODE -ne 0) { throw "Tauri NSIS build failed with exit code $LASTEXITCODE; no release manifest was written." }
    } finally {
        Pop-Location
    }

    Assert-CleanSourceTree -Root $root
    $sourceShaAfter = Get-GitValue -Root $root -Arguments @('rev-parse', 'HEAD')
    $lockHashesAfter = Get-LockHashes -Root $root
    if ($sourceShaAfter -ne $sourceSha) { throw 'HEAD changed during release build; refusing to freeze output.' }
    if ((ConvertTo-Json $lockHashesAfter -Compress) -ne (ConvertTo-Json $lockHashesBefore -Compress)) {
        throw 'A lockfile changed during release build; refusing to freeze output.'
    }

    $bundleDirectory = Join-Path $cargoTargetRoot "$($script:ReleaseTarget)/release/bundle/nsis"
    if (-not (Test-Path -LiteralPath $bundleDirectory -PathType Container)) { throw "NSIS output directory was not produced: $bundleDirectory" }
    $installerPaths = @(Get-ChildItem -LiteralPath $bundleDirectory -Filter '*.exe' -File | ForEach-Object { $_.FullName })
    $installerPath = Get-UniqueInstaller -Paths $installerPaths
    $installerMachine = Assert-NsisSetupMachine -Path $installerPath
    $applicationPath = Join-Path $cargoTargetRoot "$($script:ReleaseTarget)/release/research-workbench.exe"
    if (-not (Test-Path -LiteralPath $applicationPath -PathType Leaf)) {
        throw "The x64 Research Workbench application binary was not produced: $applicationPath"
    }
    $applicationMachine = Assert-PeX64 -Path $applicationPath

    $tools = [ordered]@{
        node = Get-ToolOutput -Command 'node.exe' -Arguments @('--version')
        npm = Get-ToolOutput -Command 'npm.cmd' -Arguments @('--version')
        rustc = (Get-ToolOutput -Command 'rustc.exe' -Arguments @('--version'))
        cargo = (Get-ToolOutput -Command 'cargo.exe' -Arguments @('--version'))
        git = Get-ToolOutput -Command 'git.exe' -Arguments @('--version')
        msvcToolset = $env:VCToolsVersion
        windowsSdk = $env:WindowsSDKVersion
    }

    New-Item -ItemType Directory -Path $releaseDirectory | Out-Null
    $destination = Join-Path $releaseDirectory $script:InstallName
    Copy-Item -LiteralPath $installerPath -Destination $destination
    $sourceInstallerHash = Get-Sha256 -Path $installerPath
    $frozenInstallerHash = Get-Sha256 -Path $destination
    if ($sourceInstallerHash -ne $frozenInstallerHash) { throw 'Frozen installer hash differs from the build output; no success manifest was written.' }

    $manifest = [ordered]@{
        schemaVersion = 1
        releaseVersion = $script:ReleaseVersion
        sourceSHA = $sourceSha
        target = $script:ReleaseTarget
        configuration = [ordered]@{
            installer = 'NSIS'
            installMode = $tauri.bundle.windows.nsis.installMode
            webviewInstallMode = $tauri.bundle.windows.webviewInstallMode.type
        }
        buildCommand = $command
        buildOutputPath = $installerPath.Substring($commonRepositoryRoot.TrimEnd('\').Length + 1).Replace('\', '/')
        frozenArtifact = $script:InstallName
        artifact = [ordered]@{
            sha256 = $frozenInstallerHash
            bytes = (Get-Item -LiteralPath $destination).Length
            peMachine = Format-PeMachine -Machine $installerMachine
        }
        applicationBinary = [ordered]@{
            file = 'research-workbench.exe'
            sha256 = Get-Sha256 -Path $applicationPath
            bytes = (Get-Item -LiteralPath $applicationPath).Length
            peMachine = Format-PeMachine -Machine $applicationMachine
        }
        lockfileSha256 = $lockHashesAfter
        tools = $tools
        results = [ordered]@{
            bundleGenerated = 'checked'
            applicationArchitecture = 'checked: x64 PE header'
            installerStubArchitecture = "checked: $(Format-PeMachine -Machine $installerMachine)"
            webView2OfflinePayload = 'pending: inspect generated NSIS payload'
            pdfAssetsInInstaller = 'pending: inspect packaged frontend assets'
            appReleaseExecuted = 'pending'
            cleanOfflineInstallation = 'pending'
            dataPreservation = 'pending'
        }
        sourceTreeCleanBeforeAndAfter = $true
        builtAtUtc = [DateTime]::UtcNow.ToString('yyyy-MM-ddTHH:mm:ssZ')
    }
    $manifestPath = Join-Path $releaseDirectory 'manifest.json'
    $manifestTempPath = "$manifestPath.tmp"
    [IO.File]::WriteAllText($manifestTempPath, (ConvertTo-Json -InputObject $manifest -Depth 8), [Text.UTF8Encoding]::new($false))
    Move-Item -LiteralPath $manifestTempPath -Destination $manifestPath
    Write-Output "Frozen pilot installer: $destination"
    Write-Output "Installer SHA-256: $frozenInstallerHash"
    Write-Output "Manifest: $manifestPath"
}

if ($MyInvocation.InvocationName -ne '.') {
    Invoke-ReleaseBuild
}
