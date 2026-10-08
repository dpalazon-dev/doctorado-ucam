# Dot-source this script to configure only the current development process.
$ErrorActionPreference='Stop'
$repoRoot=Split-Path $PSScriptRoot -Parent
$cargoBin=Join-Path $env:USERPROFILE '.cargo/bin'
if(!(Test-Path -LiteralPath (Join-Path $cargoBin 'cargo.exe'))){throw 'Rust MSVC is required; see docs/development/ENVIRONMENT.md'}
$env:PATH=$cargoBin+';'+$env:PATH
if(!$env:VCINSTALLDIR){
 $vswherePath=Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio/Installer/vswhere.exe'
 if(!(Test-Path -LiteralPath $vswherePath)){throw 'Visual Studio C++ build tools are required'}
 $vsInstall=& $vswherePath -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
 if(!$vsInstall){throw 'No compatible Visual Studio C++ installation found'}
 $vcvars=Join-Path $vsInstall 'VC/Auxiliary/Build/vcvars64.bat'
 $environment=& $env:COMSPEC /d /c ('call "'+$vcvars+'" >nul && set')
 if($LASTEXITCODE -ne 0){throw 'Could not enter the x64 MSVC environment'}
 foreach($line in $environment){if($line -match '^([^=]+)=(.*)$'){[Environment]::SetEnvironmentVariable($matches[1],$matches[2],'Process')}}
}
if(!$env:CARGO_TARGET_DIR){
 $gitCommon=& git -C $repoRoot rev-parse --path-format=absolute --git-common-dir
 if($LASTEXITCODE -ne 0){throw 'Could not locate the repository build cache'}
 $repositoryRoot=Split-Path $gitCommon -Parent
 $env:CARGO_TARGET_DIR=Join-Path $repositoryRoot 'work/cargo-target'
}
