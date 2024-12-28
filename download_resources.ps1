# Create resources directory
New-Item -ItemType Directory -Force -Path "assets"

# Create valid move indicator
$circle = @"
<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
  <circle cx="50" cy="50" r="40" fill="rgba(0,255,0,0.3)" stroke="none"/>
</svg>
"@
$circle | Out-File "assets/valid_move.svg" -Encoding UTF8

# Convert SVG to PNG using ImageMagick
& "C:\Program Files\ImageMagick-7.1.1-Q16-HDRI\magick.exe" convert "assets/valid_move.svg" "assets/valid_move.png"

# Create piece SVGs and convert to PNGs
$pieces = @{
    "white_king" = @"
<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
  <circle cx="50" cy="50" r="45" fill="white" stroke="black" stroke-width="2"/>
  <text x="50" y="70" font-size="60" text-anchor="middle" fill="black">♔</text>
</svg>
"@
    "white_queen" = @"
<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
  <circle cx="50" cy="50" r="45" fill="white" stroke="black" stroke-width="2"/>
  <text x="50" y="70" font-size="60" text-anchor="middle" fill="black">♕</text>
</svg>
"@
    "white_rook" = @"
<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
  <circle cx="50" cy="50" r="45" fill="white" stroke="black" stroke-width="2"/>
  <text x="50" y="70" font-size="60" text-anchor="middle" fill="black">♖</text>
</svg>
"@
    "white_bishop" = @"
<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
  <circle cx="50" cy="50" r="45" fill="white" stroke="black" stroke-width="2"/>
  <text x="50" y="70" font-size="60" text-anchor="middle" fill="black">♗</text>
</svg>
"@
    "white_knight" = @"
<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
  <circle cx="50" cy="50" r="45" fill="white" stroke="black" stroke-width="2"/>
  <text x="50" y="70" font-size="60" text-anchor="middle" fill="black">♘</text>
</svg>
"@
    "white_pawn" = @"
<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
  <circle cx="50" cy="50" r="45" fill="white" stroke="black" stroke-width="2"/>
  <text x="50" y="70" font-size="60" text-anchor="middle" fill="black">♙</text>
</svg>
"@
    "black_king" = @"
<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
  <circle cx="50" cy="50" r="45" fill="black" stroke="white" stroke-width="2"/>
  <text x="50" y="70" font-size="60" text-anchor="middle" fill="white">♔</text>
</svg>
"@
    "black_queen" = @"
<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
  <circle cx="50" cy="50" r="45" fill="black" stroke="white" stroke-width="2"/>
  <text x="50" y="70" font-size="60" text-anchor="middle" fill="white">♕</text>
</svg>
"@
    "black_rook" = @"
<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
  <circle cx="50" cy="50" r="45" fill="black" stroke="white" stroke-width="2"/>
  <text x="50" y="70" font-size="60" text-anchor="middle" fill="white">♖</text>
</svg>
"@
    "black_bishop" = @"
<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
  <circle cx="50" cy="50" r="45" fill="black" stroke="white" stroke-width="2"/>
  <text x="50" y="70" font-size="60" text-anchor="middle" fill="white">♗</text>
</svg>
"@
    "black_knight" = @"
<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
  <circle cx="50" cy="50" r="45" fill="black" stroke="white" stroke-width="2"/>
  <text x="50" y="70" font-size="60" text-anchor="middle" fill="white">♘</text>
</svg>
"@
    "black_pawn" = @"
<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
  <circle cx="50" cy="50" r="45" fill="black" stroke="white" stroke-width="2"/>
  <text x="50" y="70" font-size="60" text-anchor="middle" fill="white">♙</text>
</svg>
"@
}

# Save all pieces and convert to PNG
foreach ($piece in $pieces.Keys) {
    $pieces[$piece] | Out-File "assets/$piece.svg" -Encoding UTF8
    & "C:\Program Files\ImageMagick-7.1.1-Q16-HDRI\magick.exe" convert "assets/$piece.svg" "assets/$piece.png"
    Remove-Item "assets/$piece.svg"
    Write-Host "Created $piece.png"
}

Remove-Item "assets/valid_move.svg"
Write-Host "All resources created!" 