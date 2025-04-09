#!/usr/bin/env python3

def hex_to_decimal(hex_str):
    """Convert a hex string to a decimal string."""
    # Remove '0x' prefix if present
    if hex_str.startswith('0x'):
        hex_str = hex_str[2:]
    
    # Convert to decimal
    decimal = int(hex_str, 16)
    return str(decimal)

# Example values from the README
g1_x_hex = "0x2c1619993b1ae6dcb33661d64742b2b7336a90c3db7dfaba6eb691d98fea060a"
g1_y_hex = "0x0a16f975b962fecbe821b85c2d96093a9db1f2cf12b878a2376d99a16c4d9f06"
scalar_hex = "0xffe3be6f94645e9216938adbaa5e621cd4afd69ffd75fb433498ca18866b248c"

# Convert to decimal
g1_x_decimal = hex_to_decimal(g1_x_hex)
g1_y_decimal = hex_to_decimal(g1_y_hex)
scalar_decimal = hex_to_decimal(scalar_hex)

print(f"G1_X (hex): {g1_x_hex}")
print(f"G1_X (decimal): {g1_x_decimal}")
print()
print(f"G1_Y (hex): {g1_y_hex}")
print(f"G1_Y (decimal): {g1_y_decimal}")
print()
print(f"SCALAR (hex): {scalar_hex}")
print(f"SCALAR (decimal): {scalar_decimal}") 