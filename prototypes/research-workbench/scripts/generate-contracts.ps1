param([switch]$Check)
$ErrorActionPreference='Stop'
. (Join-Path $PSScriptRoot 'development-env.ps1')
$repoRoot=Split-Path $PSScriptRoot -Parent
$cargoPath=Join-Path $env:USERPROFILE '.cargo/bin/cargo.exe'
Push-Location $repoRoot
try {
 $lines=& $cargoPath run --quiet --manifest-path src-tauri/Cargo.toml --bin generate_contracts
 if($LASTEXITCODE -ne 0){exit $LASTEXITCODE}
 $generated=($lines -join "`n")+"`n"
 $rustSource=[IO.File]::ReadAllText((Join-Path $repoRoot 'src-tauri/src/transport/dto.rs'))
 foreach($dto in [regex]::Matches($rustSource,'pub (?:struct|enum) (\w+)')){if($generated -notmatch ('export type '+$dto.Groups[1].Value+'[ <=>]')){throw ('Unexported Rust DTO: '+$dto.Groups[1].Value)}}
 $target=Join-Path $repoRoot 'src/shared/contracts/generated/contracts.ts'
 if($Check){if(!(Test-Path -LiteralPath $target) -or [IO.File]::ReadAllText($target).Replace("`r`n","`n") -cne $generated){throw 'Generated contracts drift; run scripts/generate-contracts.ps1'}}
 else {[IO.Directory]::CreateDirectory((Split-Path $target -Parent))|Out-Null;[IO.File]::WriteAllText($target,$generated,[Text.UTF8Encoding]::new($false))}
}finally{Pop-Location}
