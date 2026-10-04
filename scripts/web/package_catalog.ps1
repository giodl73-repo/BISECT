param(
    [string]$Site = 'dist/precomputed-lab',
    [string]$Archive = 'dist/lab-catalog.tar.gz'
)
$catalogRoot = (Resolve-Path $Site).Path
node (Join-Path $PSScriptRoot 'verify_catalog.mjs') $catalogRoot
if ($LASTEXITCODE -ne 0) { throw 'Catalog validation failed.' }
if (Test-Path -LiteralPath $Archive) { throw 'Archive already exists; choose a new filename.' }
tar -czf $Archive -C $catalogRoot .
if ($LASTEXITCODE -ne 0) { throw 'Catalog archive creation failed.' }
$catalogHash = (Get-FileHash -LiteralPath $Archive -Algorithm SHA256).Hash.ToLowerInvariant()
[IO.File]::WriteAllText("$Archive.sha256", "$catalogHash  $([IO.Path]::GetFileName($Archive))`n")
Write-Output "Archive: $Archive"
Write-Output "SHA-256: $catalogHash"
