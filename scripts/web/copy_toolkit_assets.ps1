param([Parameter(Mandatory=$true)][string]$Output)
$ErrorActionPreference = 'Stop'
$browserAssets = Join-Path $PSScriptRoot '../../web/lab'
New-Item -ItemType Directory -Path $Output -Force | Out-Null
Get-ChildItem -LiteralPath $browserAssets -File | Where-Object { $_.Extension -in @('.js', '.css') } | ForEach-Object {
    Copy-Item -LiteralPath $_.FullName -Destination (Join-Path $Output $_.Name)
}
Copy-Item -LiteralPath (Join-Path $browserAssets 'workbench.html') -Destination (Join-Path $Output 'index.html')
