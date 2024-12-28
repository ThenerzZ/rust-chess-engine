# Create release directory
$releaseDir = "release-1.0.0"
New-Item -ItemType Directory -Force -Path $releaseDir

# Copy the executable
Copy-Item "target/release/chess-engine.exe" -Destination "$releaseDir/"

# Copy required assets and resources
Copy-Item -Path "assets" -Destination "$releaseDir/" -Recurse
Copy-Item -Path "resources" -Destination "$releaseDir/" -Recurse

# Copy documentation
Copy-Item "README-1.0.0.md" -Destination "$releaseDir/README.md"
Copy-Item "LICENSE" -Destination "$releaseDir/"

# Create a launcher script
@"
@echo off
start "" "%~dp0chess-engine.exe"
"@ | Out-File -FilePath "$releaseDir/launch.bat" -Encoding ASCII

# Create ZIP archive
Compress-Archive -Path "$releaseDir/*" -DestinationPath "chess-engine-v1.0.0-windows.zip" -Force

Write-Host "Release package created in $releaseDir"
Write-Host "ZIP archive created: chess-engine-v1.0.0-windows.zip" 