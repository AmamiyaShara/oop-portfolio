@echo off
setlocal
cd /d "%~dp0.."

powershell -NoProfile -ExecutionPolicy Bypass -Command "$client = New-Object Net.Sockets.TcpClient; try { $client.Connect('127.0.0.1', 4173); $running = $true } catch { $running = $false } finally { $client.Dispose() }; if (-not $running) { Start-Process -FilePath 'py' -ArgumentList '-m','http.server','4173' -WorkingDirectory (Get-Location) -WindowStyle Hidden; Start-Sleep -Milliseconds 900 }; Start-Process 'http://127.0.0.1:4173/m3-creature/'"

endlocal
