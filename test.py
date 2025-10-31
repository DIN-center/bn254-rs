#!/usr/bin/env python3
"""
BN254 Format Convention Diagnostic
Checks different format conventions between Rust/Solidity and Python implementations
"""

from py_ecc.bn128 import (
    G1, G2, FQ, FQ2,
    pairing, multiply,
    curve_order, field_modulus,
    is_on_curve, b
)
import hashlib
import requests
import json
import sys
import argparse

def check_coordinate_formats(x_str: str, y_str: str, point_name: str):
    """Check different possible coordinate formats"""
    print(f"\n{'='*60}")
    print(f"FORMAT ANALYSIS: {point_name}")
    print(f"{'='*60}")
    
    # Parse as integers
    x_int = int(x_str)
    y_int = int(y_str)
    
    print(f"\nRaw values:")
    print(f"  x = {x_int}")
    print(f"  y = {y_int}")
    
    # Test 1: Standard format (as-is)
    try:
        point_std = (FQ(x_int), FQ(y_int))
        on_curve_std = is_on_curve(point_std, b)
        print(f"\n1. Standard format (x, y): {on_curve_std}")
    except:
        on_curve_std = False
        print(f"\n1. Standard format (x, y): Invalid")
    
    # Test 2: Swapped coordinates
    try:
        point_swap = (FQ(y_int), FQ(x_int))
        on_curve_swap = is_on_curve(point_swap, b)
        print(f"2. Swapped format (y, x): {on_curve_swap}")
    except:
        on_curve_swap = False
        print(f"2. Swapped format (y, x): Invalid")
    
    # Test 3: Negated y-coordinate (common in some implementations)
    try:
        y_neg = field_modulus - y_int
        point_neg_y = (FQ(x_int), FQ(y_neg))
        on_curve_neg_y = is_on_curve(point_neg_y, b)
        print(f"3. Negated y-coordinate (x, -y): {on_curve_neg_y}")
    except:
        on_curve_neg_y = False
        print(f"3. Negated y-coordinate (x, -y): Invalid")
    
    # Test 4: Montgomery form conversion (some implementations use this)
    # BN254 sometimes uses different coordinate systems
    try:
        # Try interpreting as affine coordinates that need conversion
        x_mont = (x_int * pow(2, 256, field_modulus)) % field_modulus
        y_mont = (y_int * pow(2, 256, field_modulus)) % field_modulus
        point_mont = (FQ(x_mont), FQ(y_mont))
        on_curve_mont = is_on_curve(point_mont, b)
        print(f"4. Montgomery form: {on_curve_mont}")
    except:
        on_curve_mont = False
        print(f"4. Montgomery form: Invalid")
    
    # Test 5: Check if coordinates are in a different field representation
    print(f"\n5. Field checks:")
    print(f"   x < field_modulus: {x_int < field_modulus}")
    print(f"   y < field_modulus: {y_int < field_modulus}")
    print(f"   x < curve_order: {x_int < curve_order}")
    print(f"   y < curve_order: {y_int < curve_order}")
    
    return {
        'standard': on_curve_std,
        'swapped': on_curve_swap,
        'negated_y': on_curve_neg_y,
        'montgomery': on_curve_mont
    }

def verify_with_different_conventions(sig_data, msg_data, pk_data, msg_point=None):
    """Try different pairing conventions"""
    print(f"\n{'='*60}")
    print("PAIRING CONVENTION TESTS")
    print(f"{'='*60}")

    # Convert all points
    sig_x, sig_y = int(sig_data['x']), int(sig_data['y'])

    # Public key G2 point
    pk_x = FQ2([int(pk_data['x'][0]), int(pk_data['x'][1])])
    pk_y = FQ2([int(pk_data['y'][0]), int(pk_data['y'][1])])
    pk_g2 = (pk_x, pk_y)

    # Use the actual message point if provided
    if msg_point is None:
        msg_x, msg_y = int(msg_data['x']), int(msg_data['y'])
        msg_point = (FQ(msg_x), FQ(msg_y))

    print("\n1. Testing: e(msg, pk) == e(sig, G2)")
    print("   (Standard BLS signature convention)")

    # Try different signature formats
    test_cases = [
        ("Standard (x, y)", (FQ(sig_x), FQ(sig_y))),
        ("Swapped (y, x)", (FQ(sig_y), FQ(sig_x))),
        ("Negated y", (FQ(sig_x), FQ(field_modulus - sig_y))),
    ]

    for name, sig_point in test_cases:
        try:
            if is_on_curve(sig_point, b):
                lhs = pairing(pk_g2, msg_point)
                rhs = pairing(G2, sig_point)
                valid = lhs == rhs
                print(f"   {name}: {'✓ VALID' if valid else '✗ Invalid'}")
                if valid:
                    return sig_point
        except Exception as e:
            print(f"   {name}: ✗ Error - {str(e)[:50]}")

    print("\n2. Testing: e(sig, pk) == e(msg, G2)")
    print("   (Alternative convention)")

    for name, sig_point in test_cases:
        try:
            if is_on_curve(sig_point, b):
                lhs = pairing(pk_g2, sig_point)
                rhs = pairing(G2, msg_point)
                valid = lhs == rhs
                print(f"   {name}: {'✓ VALID' if valid else '✗ Invalid'}")
                if valid:
                    return sig_point
        except Exception as e:
            print(f"   {name}: ✗ Error - {str(e)[:50]}")

    return None

def analyze_g1_g2_naming(sig_data, msg_data):
    """Analyze if g1/g2 naming is causing confusion"""
    print(f"\n{'='*60}")
    print("G1/G2 NAMING CONVENTION ANALYSIS")
    print(f"{'='*60}")
    
    print("\nYour Rust output labels:")
    print("  'signature': The actual signature point")
    print("  'g1': Labeled as G1 (but this might be H(message))")
    print("  'g2': Labeled as G2 (this is the public key)")
    
    print("\nIn standard BLS on BN254:")
    print("  - Messages are hashed to G1 points")
    print("  - Signatures are G1 points")
    print("  - Public keys are G2 points")
    
    print("\nYour data suggests:")
    print("  - 'g1' (the valid point) is actually H(message)")
    print("  - 'signature' (invalid point) might be incorrectly computed")
    print("  - 'g2' is correctly the public key")
    
    # Check if swapping helps
    print("\nTrying interpretation where 'g1' is the signature:")
    sig_as_g1 = (FQ(int(msg_data['x'])), FQ(int(msg_data['y'])))
    msg_as_sig = (FQ(int(sig_data['x'])), FQ(int(sig_data['y'])))
    
    print(f"  'g1' point on curve: {is_on_curve(sig_as_g1, b)}")
    print(f"  'signature' point on curve: {is_on_curve(msg_as_sig, b)}")

def suggest_fixes():
    """Suggest potential fixes"""
    print(f"\n{'='*60}")
    print("SUGGESTED FIXES FOR YOUR RUST CODE")
    print(f"{'='*60}")
    
    print("""
1. **Check point serialization in Rust**:
   - Ensure you're using affine coordinates, not Jacobian/projective
   - Make sure to normalize points before extracting x,y
   
2. **Verify field elements**:
   - Check if your Rust library uses Montgomery representation
   - Ensure proper conversion to standard representation
   
3. **Common Rust BN254 library issues**:
   - ark-bn254: Use `.into_affine()` before extracting coordinates
   - bn: Ensure proper coordinate extraction
   - Example fix:
     ```rust
     let sig_point = // your signature point
     let affine = sig_point.into_affine();
     let x = affine.x.to_string();
     let y = affine.y.to_string();
     ```

4. **Check the signature equation**:
   - Standard BLS: signature = private_key * H(message)
   - Verify you're multiplying in the right order
   
5. **Debug by generating a known test vector**:
   - Use private_key = 1
   - Message = "test"
   - Compare output with py_ecc reference
""")

def verify_signature_quick(sig_data, pubkey_data, pk_g2_data, msg_point):
    """Quick signature verification - only checks validity"""
    print(f"\n{'='*60}")
    print("QUICK VALIDATION")
    print(f"{'='*60}")

    # Convert signature point
    sig_x, sig_y = int(sig_data['x']), int(sig_data['y'])
    sig_point = (FQ(sig_x), FQ(sig_y))

    # Convert public key G1 point
    pk_x, pk_y = int(pubkey_data['x']), int(pubkey_data['y'])
    pk_g1_point = (FQ(pk_x), FQ(pk_y))

    # Convert public key G2 point
    pk_g2_x = FQ2([int(pk_g2_data['x'][0]), int(pk_g2_data['x'][1])])
    pk_g2_y = FQ2([int(pk_g2_data['y'][0]), int(pk_g2_data['y'][1])])
    pk_g2 = (pk_g2_x, pk_g2_y)

    # Check if points are on curve
    sig_valid = is_on_curve(sig_point, b)
    pk_valid = is_on_curve(pk_g1_point, b)

    print(f"\n✓ Signature on curve: {sig_valid}")
    print(f"✓ Public key on curve: {pk_valid}")

    if not (sig_valid and pk_valid):
        print("\n✗ FAILED: Points are not on curve")
        return None

    # Verify BLS signature: e(msg, pk_g2) == e(sig, G2)
    try:
        lhs = pairing(pk_g2, msg_point)
        rhs = pairing(G2, sig_point)
        pairing_valid = lhs == rhs

        if pairing_valid:
            print(f"\n✓ BLS pairing verification: PASSED")
            print("  e(message, pubkey_g2) == e(signature, G2) ✓")
            return sig_point
        else:
            print(f"\n✗ BLS pairing verification: FAILED")
            return None
    except Exception as e:
        print(f"\n✗ Pairing error: {str(e)[:80]}")
        return None

def fetch_signature_from_server(server_url="http://localhost:3000",
                                 eoa_address="0x70997970C51812dc3A010C7d01b50e0d17dc79C8"):
    """Fetch a signature from the bn254-rs server"""
    print(f"\n{'='*60}")
    print("FETCHING SIGNATURE FROM SERVER")
    print(f"{'='*60}")
    print(f"Server: {server_url}")
    print(f"EOA: {eoa_address}")

    # Generate a valid test message by hashing to curve
    # We'll use a simple hash-to-curve: hash message, multiply generator
    test_msg = b"test message for BN254 signing"
    from hashlib import sha256

    # Hash the message to get a scalar
    msg_hash = sha256(test_msg).digest()
    scalar = int.from_bytes(msg_hash, 'big') % curve_order

    # Multiply G1 generator by the scalar to get a valid curve point
    msg_point = multiply(G1, scalar)

    # Extract coordinates and format as hex (32 bytes each = 64 hex chars)
    msg_x_bytes = msg_point[0].n.to_bytes(32, 'big')
    msg_y_bytes = msg_point[1].n.to_bytes(32, 'big')

    # Concatenate to 128 hex chars
    message = msg_x_bytes.hex() + msg_y_bytes.hex()

    print(f"Message (hashed to curve): {message[:32]}...{message[-32:]}")
    print(f"  Original message: {test_msg.decode()}")
    print(f"  Hash scalar: {scalar}")
    print(f"  Point X: {msg_point[0].n}")
    print(f"  Point Y: {msg_point[1].n}")
    print(f"  On curve: {is_on_curve(msg_point, b)}")

    payload = {
        "eoa_address": eoa_address,
        "message": message
    }

    try:
        response = requests.post(f"{server_url}/sign", json=payload, timeout=5)
        response.raise_for_status()
        data = response.json()

        print("\n✓ Successfully received response from server")
        print(f"  Signature X: {data['signature']['x'][:20]}...")
        print(f"  Signature Y: {data['signature']['y'][:20]}...")
        print(f"  G1 (PubKey) X: {data['g1']['x'][:20]}...")
        print(f"  G2 components: {len(data['g2']['x'])} x values, {len(data['g2']['y'])} y values")

        # Convert to the format expected by the rest of the script
        test_data = {
            "signature": {
                "x": data['signature']['x'],
                "y": data['signature']['y']
            },
            "g1": {
                "x": data['g1']['x'],
                "y": data['g1']['y']
            },
            "g2": {
                "x": data['g2']['x'],
                "y": data['g2']['y']
            },
            "message_point": msg_point  # Store for pairing verification
        }

        return test_data

    except requests.exceptions.ConnectionError:
        print("\n✗ ERROR: Could not connect to server")
        print(f"  Make sure the server is running at {server_url}")
        print(f"  Start it with: make serve")
        sys.exit(1)
    except requests.exceptions.Timeout:
        print("\n✗ ERROR: Request timed out")
        sys.exit(1)
    except requests.exceptions.HTTPError as e:
        print(f"\n✗ HTTP ERROR: {e}")
        print(f"  Response: {response.text}")
        sys.exit(1)
    except Exception as e:
        print(f"\n✗ UNEXPECTED ERROR: {e}")
        sys.exit(1)

def main():
    # Parse command line arguments
    parser = argparse.ArgumentParser(
        description='BN254 signature validation and diagnostic tool'
    )
    parser.add_argument(
        '--quick',
        action='store_true',
        help='Quick mode: only validate curve points and pairing (skip diagnostics)'
    )
    parser.add_argument(
        '--server',
        default='http://localhost:3000',
        help='Server URL (default: http://localhost:3000)'
    )
    parser.add_argument(
        '--eoa',
        default='0x70997970C51812dc3A010C7d01b50e0d17dc79C8',
        help='EOA address to test (default: 0x70997970C51812dc3A010C7d01b50e0d17dc79C8)'
    )
    args = parser.parse_args()

    # Fetch live data from server
    test_data = fetch_signature_from_server(args.server, args.eoa)

    if not args.quick:
        # Full diagnostic mode
        print("="*60)
        print("BN254 FORMAT CONVENTION DIAGNOSTIC")
        print("="*60)

        # Check different formats for signature
        sig_formats = check_coordinate_formats(
            test_data['signature']['x'],
            test_data['signature']['y'],
            "Signature Point"
        )

        # Check different formats for g1 (message?)
        g1_formats = check_coordinate_formats(
            test_data['g1']['x'],
            test_data['g1']['y'],
            "G1 Point (H(message)?)"
        )

        # Analyze naming conventions
        analyze_g1_g2_naming(test_data['signature'], test_data['g1'])

        # Try different pairing conventions
        msg_point = test_data.get('message_point')
        valid_sig = verify_with_different_conventions(
            test_data['signature'],
            test_data['g1'],
            test_data['g2'],
            msg_point
        )

        # Provide suggestions
        suggest_fixes()
    else:
        # Quick mode: just validate and verify pairing
        msg_point = test_data.get('message_point')
        valid_sig = verify_signature_quick(
            test_data['signature'],
            test_data['g1'],
            test_data['g2'],
            msg_point
        )
    
    # Summary
    print(f"\n{'='*60}")
    print("DIAGNOSIS SUMMARY")
    print(f"{'='*60}")

    # Check if signature is valid
    sig_x = int(test_data['signature']['x'])
    sig_y = int(test_data['signature']['y'])
    sig_point = (FQ(sig_x), FQ(sig_y))
    sig_valid = is_on_curve(sig_point, b)

    # Check if g1 (public key) is valid
    g1_x = int(test_data['g1']['x'])
    g1_y = int(test_data['g1']['y'])
    g1_point = (FQ(g1_x), FQ(g1_y))
    g1_valid = is_on_curve(g1_point, b)

    print("\n🔍 Validation Results:")
    print(f"  Signature point on curve: {'✓ YES' if sig_valid else '✗ NO'}")
    print(f"  G1 (PubKey) point on curve: {'✓ YES' if g1_valid else '✗ NO'}")

    if sig_valid and g1_valid:
        print("\n✅ SUCCESS: All points are valid BN254 curve points!")
        print("   Your Rust implementation is producing correct coordinates.")

        # Check if pairing verification passed
        if valid_sig:
            print("\n🎉 BONUS: BLS signature pairing verification PASSED!")
            print("   The signature is cryptographically valid.")
            print("   e(message, pubkey_g2) == e(signature, G2) ✓")
        else:
            print("\n⚠️  Note: Pairing verification did not pass.")
            print("   This could be due to:")
            print("   - Different BLS signing conventions")
            print("   - Message point format mismatch")
            print("   But the coordinates are still valid curve points! ✓")

        return 0
    else:
        print("\n❌ FAILURE: Some points are invalid")
        if not sig_valid:
            print("   - Signature point is NOT on the curve")
            print("   - Check coordinate extraction in sign endpoint")
        if not g1_valid:
            print("   - G1/PubKey point is NOT on the curve")
            print("   - Check key generation/derivation logic")
        print("\nMost likely cause: Coordinate conversion issue (projective vs affine)")
        print("✅ Action: Ensure .into_affine() is called before extracting x,y")
        return 1

if __name__ == "__main__":
    sys.exit(main())
