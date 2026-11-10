param([Parameter(Mandatory)][string]$Root,[Parameter(Mandatory)][string]$Path)
$ErrorActionPreference='Stop'
$taskRoot=[IO.Path]::GetFullPath($Root)
$taskFile=[IO.Path]::GetFullPath($Path)
if(!$taskFile.StartsWith($taskRoot+[IO.Path]::DirectorySeparatorChar,[StringComparison]::OrdinalIgnoreCase)){throw 'Lock file escaped the owned test root.'}
if(!(Test-Path -LiteralPath (Join-Path $taskRoot 'owned-test-marker.json'))){throw 'Missing test ownership marker.'}
$taskStream=$null
try {
    $taskStream=[IO.File]::Open($taskFile,[IO.FileMode]::Open,[IO.FileAccess]::ReadWrite,[IO.FileShare]::Read)
    @{pid=$PID;file=$taskFile;share='Read';write_denied_to_other_handles=$true} | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $taskRoot 'lock-ready.json') -Encoding utf8
    $taskDeadline=[DateTime]::UtcNow.AddSeconds(60)
    while(!(Test-Path -LiteralPath (Join-Path $taskRoot 'release-lock')) -and [DateTime]::UtcNow -lt $taskDeadline){Start-Sleep -Milliseconds 100}
} finally {
    if($null -ne $taskStream){$taskStream.Dispose()}
}
