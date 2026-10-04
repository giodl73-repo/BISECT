param(
    [int]$Port = 4317,
    [switch]$SkipBuild
)

$labRoot = (Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
Push-Location $labRoot
try {
    if (-not $SkipBuild) {
        cargo build -p bisect-cli -p bisect-web --locked
        if ($LASTEXITCODE -ne 0) { throw 'The Rust laboratory build failed.' }
    }
    & (Join-Path $labRoot 'target/debug/bisect-web.exe') --root $labRoot --port $Port
    if ($LASTEXITCODE -ne 0) { throw 'The laboratory server stopped with an error.' }
}
finally {
    Pop-Location
}
