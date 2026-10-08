$ErrorActionPreference='Stop'
Add-Type -Path (Join-Path $PSScriptRoot 'HandleProbe.cs')
$fixture=Join-Path $PSScriptRoot ('fixture-'+[Guid]::NewGuid().ToString('N'))
[System.IO.Directory]::CreateDirectory($fixture) | Out-Null
# Retain evidence; no recursive cleanup follows junctions.
[HandleProbe]::Run($fixture)
