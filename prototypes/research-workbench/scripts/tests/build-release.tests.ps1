$ErrorActionPreference = 'Stop'
$scriptPath = Join-Path $PSScriptRoot '..\build-release.ps1'
if (-not (Test-Path -LiteralPath $scriptPath)) {
    throw 'Release behavior checks failed: scripts/build-release.ps1 is missing.'
}

. $scriptPath

function Assert-Throws([scriptblock]$Action, [string]$Message) {
    $threw = $false
    try { & $Action } catch { $threw = $true }
    if (-not $threw) { throw $Message }
}

$failures = [Collections.Generic.List[string]]::new()
function Check([scriptblock]$Action, [string]$Name) {
    try { & $Action } catch { $script:failures.Add(($Name + ': ' + $_.Exception.Message)) }
}

$one = Get-UniqueInstaller -Paths @('C:\build\Research-Workbench.exe')
if ($one -ne 'C:\build\Research-Workbench.exe') {
    throw 'A single build output must be selected unchanged.'
}
Assert-Throws { Get-UniqueInstaller -Paths @() } 'A missing installer must fail.'
Assert-Throws { Get-UniqueInstaller -Paths @('a.exe', 'b.exe') } 'Ambiguous installer output must fail.'

Check { Assert-RustcHost -Details "release 1.99.0`nhost: x86_64-pc-windows-msvc`nLLVM version" } 'LF host output'
Check { Assert-RustcHost -Details "release 1.99.0`r`nhost: x86_64-pc-windows-msvc`r`nLLVM version" } 'CRLF host output'
Check {
    Assert-Throws {
        Assert-RustcHost -Details "release 1.99.0`r`nhost: aarch64-pc-windows-msvc`r`nLLVM version"
    } 'An incorrect Rust host must fail.'
} 'Incorrect host rejection'

Check {
    . (Join-Path $PSScriptRoot '..\development-env.ps1')
    $realDetails = Get-ToolOutput -Command 'rustc.exe' -Arguments @('-vV')
    Assert-RustcHost -Details $realDetails
} 'Installed rustc host probe'

$tempExe = Join-Path ([IO.Path]::GetTempPath()) ('rw-release-pe-' + [guid]::NewGuid().ToString('N') + '.exe')
try {
    $bytes = [byte[]]::new(256)
    $bytes[0] = 0x4d; $bytes[1] = 0x5a
    $bytes[0x3c] = 0x40
    $bytes[0x40] = 0x50; $bytes[0x41] = 0x45
    $bytes[0x44] = 0x64; $bytes[0x45] = 0x86
    [IO.File]::WriteAllBytes($tempExe, $bytes)
    $applicationMachine = Assert-PeX64 -Path $tempExe
    if ($applicationMachine -ne 0x8664) { throw 'The application binary must be x64.' }

    # NSIS can use a 32-bit setup stub while carrying the x64 app.
    $bytes[0x44] = 0x4c; $bytes[0x45] = 0x01
    [IO.File]::WriteAllBytes($tempExe, $bytes)
    Check {
        $setupMachine = Assert-NsisSetupMachine -Path $tempExe
        if ($setupMachine -ne 0x014c) { throw 'The NSIS setup stub machine should be recorded as i386.' }
        if ((Format-PeMachine -Machine $setupMachine) -ne 'i386 (0x014c)') { throw 'The setup architecture label must reflect the actual PE machine.' }
    } 'x86 NSIS setup stub acceptance'

    $bytes[0x44] = 0x64; $bytes[0x45] = 0xaa
    [IO.File]::WriteAllBytes($tempExe, $bytes)
    Check {
        if (-not (Get-Command Assert-NsisSetupMachine -ErrorAction SilentlyContinue)) { throw 'NSIS setup machine validator is missing.' }
        Assert-Throws { Assert-NsisSetupMachine -Path $tempExe } 'Unsupported NSIS setup machine must fail.'
    } 'Unsupported setup stub rejection'
} finally {
    if (Test-Path -LiteralPath $tempExe) { Remove-Item -LiteralPath $tempExe }
}

Assert-ReleaseVersionContract -PackageVersion '0.0.1' -TauriVersion '0.0.1' -CargoVersion '0.0.1'
Assert-Throws {
    Assert-ReleaseVersionContract -PackageVersion '0.0.1' -TauriVersion '0.0.1' -CargoVersion '0.1.0'
} 'A mixed version contract must fail.'

if ($failures.Count -gt 0) {
    $failureLines = @($failures | ForEach-Object { ' - ' + $_ })
    throw ("Release behavior regressions failed:`n" + ($failureLines -join [Environment]::NewLine))
}

Write-Output 'build-release behavior checks passed'
