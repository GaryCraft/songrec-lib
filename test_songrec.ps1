#!/usr/bin/env pwsh
# SongRec Test Script
# This script tests various SongRec functionality

Write-Host "🎵 SongRec Test Script 🎵" -ForegroundColor Cyan
Write-Host "=========================" -ForegroundColor Cyan

# Change to the SongRec directory
Set-Location "x:\Development\ForCompiling\SongRec"

Write-Host "`n1. Building SongRec..." -ForegroundColor Yellow
cargo build --bin songrec-cli
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Build failed!" -ForegroundColor Red
    exit 1
}
Write-Host "✅ Build successful!" -ForegroundColor Green

Write-Host "`n2. Listing available audio devices..." -ForegroundColor Yellow
cargo run --bin songrec-cli devices

Write-Host "`n3. Testing recognition with test audio (verbose)..." -ForegroundColor Yellow
cargo run --bin songrec-cli recognize test_audio.wav

Write-Host "`n4. Testing recognition with test audio (quiet mode)..." -ForegroundColor Yellow
Write-Host "Expected output: Clean, parsable format" -ForegroundColor Gray
cargo run --bin songrec-cli recognize test_audio.wav --quiet

Write-Host "`n5. Testing live audio capture (debug mode)..." -ForegroundColor Yellow
Write-Host "This will show audio levels and processing details" -ForegroundColor Gray
Write-Host "Press Ctrl+C to stop after a few seconds" -ForegroundColor Gray
Start-Sleep -Seconds 2
cargo run --bin songrec-cli listen --device "Voicemeeter Out B3 (VB-Audio Voicemeeter VAIO)" --timeout 10

Write-Host "`n🎉 SongRec is ready to use!" -ForegroundColor Green

Write-Host "`n🔧 Available commands:" -ForegroundColor Yellow
Write-Host "   📁 List devices:     cargo run --bin songrec-cli devices" -ForegroundColor White
Write-Host "   🎵 Recognize file:   cargo run --bin songrec-cli recognize test_audio.wav" -ForegroundColor White
Write-Host "   🔇 Quiet recognize:  cargo run --bin songrec-cli recognize test_audio.wav --quiet" -ForegroundColor White
Write-Host "   🎧 Listen (default): cargo run --bin songrec-cli listen" -ForegroundColor White
Write-Host "   🎧 Listen (Out B3):  cargo run --bin songrec-cli listen --device `"Voicemeeter Out B3 (VB-Audio Voicemeeter VAIO)`"" -ForegroundColor White
Write-Host "   🔇 Listen (quiet):   cargo run --bin songrec-cli listen --device `"Voicemeeter Out B3 (VB-Audio Voicemeeter VAIO)`" --quiet" -ForegroundColor White
Write-Host "   🚫 No deduplication: cargo run --bin songrec-cli listen --device `"Out B3`" --no-dedupe" -ForegroundColor White

Write-Host "`n🚀 New Features:" -ForegroundColor Cyan
Write-Host "   • Quiet mode: Use --quiet or -q for clean, parsable output" -ForegroundColor White
Write-Host "   • Deduplication: Prevents duplicate requests (use --no-dedupe to disable)" -ForegroundColor White
Write-Host "   • Multiple formats: --format json, --format csv" -ForegroundColor White

Write-Host "`n📋 Quick Start Command:" -ForegroundColor Cyan
Write-Host "cargo run --bin songrec-cli listen --device `"Voicemeeter Out B3 (VB-Audio Voicemeeter VAIO)`" --quiet" -ForegroundColor Yellow

Write-Host "`n💡 Perfect for automation and scripting!" -ForegroundColor Green
