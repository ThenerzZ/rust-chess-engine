#!/bin/bash

# Create resources directory
mkdir -p resources

# Download chess pieces from Wikimedia Commons (merida set)
pieces=("king" "queen" "rook" "bishop" "knight" "pawn")
colors=("white" "black")

for color in "${colors[@]}"; do
    for piece in "${pieces[@]}"; do
        url="https://upload.wikimedia.org/wikipedia/commons/thumb"
        case "$color" in
            "white")
                case "$piece" in
                    "king")   hash="4/42";;
                    "queen")  hash="1/15";;
                    "rook")   hash="7/72";;
                    "bishop") hash="b/b1";;
                    "knight") hash="7/70";;
                    "pawn")   hash="4/45";;
                esac
                ;;
            "black")
                case "$piece" in
                    "king")   hash="f/f0";;
                    "queen")  hash="4/47";;
                    "rook")   hash="f/ff";;
                    "bishop") hash="9/98";;
                    "knight") hash="e/ef";;
                    "pawn")   hash="c/c7";;
                esac
                ;;
        esac
        
        filename="resources/${color}_${piece}.png"
        full_url="$url/$hash/Chess_${piece}${color:0:1}t45.svg/240px-Chess_${piece}${color:0:1}t45.svg.png"
        
        echo "Downloading $filename..."
        curl -o "$filename" "$full_url"
    done
done

echo "All pieces downloaded!" 