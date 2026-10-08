$ErrorActionPreference='Stop'
. (Join-Path $PSScriptRoot 'development-env.ps1')
$repoRoot=Split-Path $PSScriptRoot -Parent
$cargoPath=Join-Path $env:USERPROFILE '.cargo/bin/cargo.exe'
Push-Location $repoRoot
try {
 foreach($task in @('typecheck','test','build')){& npm.cmd run $task;if($LASTEXITCODE -ne 0){exit $LASTEXITCODE}}
 & $cargoPath fmt --manifest-path src-tauri/Cargo.toml --check;if($LASTEXITCODE -ne 0){exit $LASTEXITCODE}
 & $cargoPath clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings;if($LASTEXITCODE -ne 0){exit $LASTEXITCODE}
 & $cargoPath test --manifest-path src-tauri/Cargo.toml;if($LASTEXITCODE -ne 0){exit $LASTEXITCODE}
 & (Join-Path $PSScriptRoot 'generate-contracts.ps1') -Check
 if($LASTEXITCODE -ne 0){exit $LASTEXITCODE}
}finally{Pop-Location}
