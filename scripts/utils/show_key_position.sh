#!/bin/bash
# Show key position: scan_code → KEY_NAME → row_col

PROFILE="$HOME/.config/keyrx/devices/099a_0638.json"

# Scan code to key name mapping (Linux evdev codes)
declare -A KEYMAP=(
    [1]="Escape"      [2]="Digit1"      [3]="Digit2"      [4]="A"
    [5]="B"           [6]="C"           [7]="D"           [8]="E"
    [9]="F"           [10]="G"          [11]="H"          [12]="Minus"
    [13]="Equal"      [14]="Backspace"  [15]="Tab"        [16]="Q"
    [17]="W"          [18]="E"          [19]="R"          [20]="T"
    [21]="Y"          [22]="U"          [23]="I"          [24]="O"
    [25]="P"          [26]="LeftBracket" [27]="RightBracket" [28]="Enter"
    [29]="LeftCtrl"   [30]="A"          [31]="S"          [32]="D"
    [33]="F"          [34]="G"          [35]="H"          [36]="J"
    [37]="K"          [38]="L"          [39]="Semicolon"  [40]="Quote"
    [41]="Backquote"  [42]="LeftShift"  [43]="Backslash"  [44]="Z"
    [45]="X"          [46]="C"          [47]="V"          [48]="B"
    [49]="N"          [50]="M"          [51]="Comma"      [52]="Period"
    [53]="Slash"      [54]="RightShift" [56]="LeftAlt"    [57]="Space"
    [58]="CapsLock"   [59]="F1"         [60]="F2"         [61]="F3"
    [62]="F4"         [63]="F5"         [64]="F6"         [65]="F7"
    [66]="F8"         [67]="F9"         [68]="F10"        [69]="NumLock"
    [87]="F11"        [88]="F12"        [92]="RightCtrl"  [93]="RightAlt"
    [94]="LeftMeta"   [99]="PrintScreen" [100]="RightMeta" [103]="Up"
    [105]="Left"      [106]="Right"     [108]="Down"      [111]="Delete"
    [119]="Pause"     [124]="Insert"    [125]="LeftSuper" [127]="Menu"
)

if [ $# -eq 0 ]; then
    echo "Usage: $0 <scan_code|all>"
    echo ""
    echo "Examples:"
    echo "  $0 4        # Show position of scan code 4"
    echo "  $0 all      # Show all mappings"
    echo "  $0 all | grep '^r3'  # Show only row 3"
    exit 1
fi

get_key_name() {
    local code=$1
    echo "${KEYMAP[$code]:-Unknown_$code}"
}

if [ "$1" = "all" ]; then
    echo "=== Keyboard Layout (Physical Order: Row → Column) ==="
    echo ""

    jq -r '.keymap | to_entries | sort_by(.value.row, .value.col) | .[] |
           "\(.value.scan_code)\t\(.value.row)\t\(.value.col)\t\(.value.alias)"' "$PROFILE" | \
    while IFS=$'\t' read -r scan row col alias; do
        key_name=$(get_key_name "$scan")
        printf "scan_code %-3s → %-15s → %s (row %s, col %s)\n" \
               "$scan" "$key_name" "$alias" "$row" "$col"
    done
else
    SCAN_CODE="$1"
    INFO=$(jq -r --arg code "$SCAN_CODE" \
        '.keymap[$code] | if . then "\(.row)\t\(.col)\t\(.alias)" else "NOT_FOUND" end' \
        "$PROFILE")

    if [ "$INFO" = "NOT_FOUND" ]; then
        echo "Scan code $SCAN_CODE not found in device profile"
        exit 1
    fi

    read -r row col alias <<< "$INFO"
    key_name=$(get_key_name "$SCAN_CODE")

    echo "scan_code $SCAN_CODE → $key_name → $alias (row $row, col $col)"
fi
