# Create resources directory
New-Item -ItemType Directory -Force -Path "resources"

# Define pieces and colors
$pieces = @("king", "queen", "rook", "bishop", "knight", "pawn")
$colors = @("white", "black")

# Hash values for each piece
$hashes = @{
    "white_king"   = "4/42"
    "white_queen"  = "1/15"
    "white_rook"   = "7/72"
    "white_bishop" = "b/b1"
    "white_knight" = "7/70"
    "white_pawn"   = "4/45"
    "black_king"   = "f/f0"
    "black_queen"  = "4/47"
    "black_rook"   = "f/ff"
    "black_bishop" = "9/98"
    "black_knight" = "e/ef"
    "black_pawn"   = "c/c7"
}

# Download each piece
foreach ($color in $colors) {
    foreach ($piece in $pieces) {
        $key = "${color}_${piece}"
        $hash = $hashes[$key]
        $filename = "resources/${key}.png"
        $url = "https://upload.wikimedia.org/wikipedia/commons/thumb/$hash/Chess_${piece}${color[0]}t45.svg/240px-Chess_${piece}${color[0]}t45.svg.png"
        
        Write-Host "Downloading $filename..."
        Invoke-WebRequest -Uri $url -OutFile $filename
    }
}

Write-Host "All pieces downloaded!" 