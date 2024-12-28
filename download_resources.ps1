# Create resources directory
New-Item -ItemType Directory -Force -Path "assets"

# Define the base URL for chess piece sprites
$baseUrl = "https://upload.wikimedia.org/wikipedia/commons/thumb"

# Define the piece URLs (Merida chess set)
$pieces = @{
    "white_king" = "4/42/Chess_klt45.svg/150px-Chess_klt45.svg.png"
    "white_queen" = "1/15/Chess_qlt45.svg/150px-Chess_qlt45.svg.png"
    "white_rook" = "7/72/Chess_rlt45.svg/150px-Chess_rlt45.svg.png"
    "white_bishop" = "b/b1/Chess_blt45.svg/150px-Chess_blt45.svg.png"
    "white_knight" = "7/70/Chess_nlt45.svg/150px-Chess_nlt45.svg.png"
    "white_pawn" = "4/45/Chess_plt45.svg/150px-Chess_plt45.svg.png"
    "black_king" = "f/f0/Chess_kdt45.svg/150px-Chess_kdt45.svg.png"
    "black_queen" = "4/47/Chess_qdt45.svg/150px-Chess_qdt45.svg.png"
    "black_rook" = "f/ff/Chess_rdt45.svg/150px-Chess_rdt45.svg.png"
    "black_bishop" = "9/98/Chess_bdt45.svg/150px-Chess_bdt45.svg.png"
    "black_knight" = "e/ef/Chess_ndt45.svg/150px-Chess_ndt45.svg.png"
    "black_pawn" = "c/c7/Chess_pdt45.svg/150px-Chess_pdt45.svg.png"
}

# Download each piece
foreach ($piece in $pieces.Keys) {
    $url = "$baseUrl/$($pieces[$piece])"
    $outFile = "assets/$piece.png"
    try {
        Invoke-WebRequest -Uri $url -OutFile $outFile
        Write-Host "Downloaded $piece.png"
    } catch {
        Write-Host "Failed to download $piece.png: $_"
    }
}

# Create valid move indicator
$validMove = @"
<svg xmlns="http://www.w3.org/2000/svg" width="150" height="150">
  <circle cx="75" cy="75" r="60" fill="rgba(0,255,0,0.3)" stroke="rgba(0,255,0,0.5)" stroke-width="4"/>
</svg>
"@
$validMove | Out-File "assets/valid_move.svg" -Encoding UTF8

# Convert valid move indicator to PNG
& "C:\Program Files\Inkscape\bin\inkscape.exe" --export-type=png --export-filename="assets/valid_move.png" "assets/valid_move.svg"
Remove-Item "assets/valid_move.svg"

Write-Host "All resources created!" 