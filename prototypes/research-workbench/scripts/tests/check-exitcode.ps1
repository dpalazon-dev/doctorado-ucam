$ErrorActionPreference='Stop'
$testRoot=Join-Path ([IO.Path]::GetTempPath()) ('research-check-'+[guid]::NewGuid())
[IO.Directory]::CreateDirectory((Join-Path $testRoot 'scripts'))|Out-Null
try {
 Copy-Item -LiteralPath (Join-Path $PSScriptRoot '../check.ps1') -Destination (Join-Path $testRoot 'scripts/check.ps1')
 # Fake only external tools; run the actual copied gate as a separate parent process.
 @'
function global:npm.cmd { $global:LASTEXITCODE=0 }
'@ | Set-Content -LiteralPath (Join-Path $testRoot 'scripts/development-env.ps1')
 # The gate resolves Cargo by absolute path. A script shim at that path overrides it.
 $gate=Get-Content -LiteralPath (Join-Path $testRoot 'scripts/check.ps1') -Raw
 $gate=$gate.Replace("`$cargoPath=Join-Path `$env:USERPROFILE '.cargo/bin/cargo.exe'","`$cargoPath=Join-Path `$PSScriptRoot 'cargo.ps1'")
 Set-Content -LiteralPath (Join-Path $testRoot 'scripts/check.ps1') -Value $gate
 'exit 0' | Set-Content -LiteralPath (Join-Path $testRoot 'scripts/cargo.ps1')
 'exit 37' | Set-Content -LiteralPath (Join-Path $testRoot 'scripts/generate-contracts.ps1')
 & powershell.exe -NoProfile -ExecutionPolicy Bypass -File (Join-Path $testRoot 'scripts/check.ps1')
 if($LASTEXITCODE -ne 37){throw "Gate hid generator exit37: observed $LASTEXITCODE"}
 Write-Output 'Parent gate propagated generator exit37.'
} finally {
 # Only the verified, task-created temp directory may be removed.
 $resolved=[IO.Path]::GetFullPath($testRoot)
 if(!$resolved.StartsWith([IO.Path]::GetFullPath([IO.Path]::GetTempPath()),[StringComparison]::OrdinalIgnoreCase)){throw 'Unexpected test path'}
 Remove-Item -LiteralPath $resolved -Recurse -Force
}
