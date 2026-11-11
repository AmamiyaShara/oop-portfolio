$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path $PSScriptRoot -Parent
$localTools = Join-Path $projectRoot '.tools'
if (Test-Path (Join-Path $localTools 'cargo/bin/cargo.exe')) {
  $env:CARGO_HOME = Join-Path $localTools 'cargo'
  $env:RUSTUP_HOME = Join-Path $localTools 'rustup'
  $cargo = Join-Path $localTools 'cargo/bin/cargo.exe'
} else { $cargo = 'cargo' }
& $cargo build --manifest-path (Join-Path $PSScriptRoot 'rust/Cargo.toml') --release --target wasm32-unknown-unknown
if ($LASTEXITCODE -ne 0) { throw 'Rust build failed' }
New-Item -ItemType Directory -Force -Path (Join-Path $PSScriptRoot 'pkg') | Out-Null
Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'rust/target/wasm32-unknown-unknown/release/residual_city.wasm') -Destination (Join-Path $PSScriptRoot 'pkg/residual_city.wasm')
