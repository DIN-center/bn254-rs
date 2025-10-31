// SPDX-License-Identifier: MIT
pragma solidity ^0.8.27;

import "./BN254.sol";

/**
 * @title BN254 Key Validator
 * @notice Validates BN254 BLS keypairs using the pairing precompile
 * @dev Uses the property that for a valid keypair (sk, pk_g1, pk_g2):
 *      e(pk_g1, G2) == e(G1, pk_g2)
 *      where pk_g1 = sk * G1 and pk_g2 = sk * G2
 */
contract BN254KeyValidator {
    using BN254 for *;

    /**
     * @notice Validates that a G1 and G2 public key pair were derived from the same private key
     * @param pubkey_g1 The public key on G1
     * @param pubkey_g2 The public key on G2
     * @return isValid True if the keypair is valid (both derived from same secret)
     *
     * @dev This uses the pairing check: e(pubkey_g1, G2) == e(G1, pubkey_g2)
     *
     *      Mathematical reasoning:
     *      - If pubkey_g1 = sk * G1 and pubkey_g2 = sk * G2
     *      - Then e(sk * G1, G2) = e(G1, G2)^sk (bilinearity)
     *      - And e(G1, sk * G2) = e(G1, G2)^sk (bilinearity)
     *      - Therefore e(pubkey_g1, G2) = e(G1, pubkey_g2)
     *
     *      The pairing precompile checks if: e(a1, a2) * e(b1, b2) == 1
     *      We rewrite our check as: e(pubkey_g1, G2) * e(G1, -pubkey_g2) == 1
     */
    function validateKeyPair(
        BN254.G1Point calldata pubkey_g1,
        BN254.G2Point calldata pubkey_g2
    ) external view returns (bool isValid) {
        // Get generator points
        BN254.G1Point memory g1_gen = BN254.generatorG1();
        BN254.G2Point memory g2_gen = BN254.generatorG2();

        // We need to check: e(pubkey_g1, G2) == e(G1, pubkey_g2)
        // The pairing precompile checks: e(a1, a2) * e(b1, b2) == 1
        // So we rearrange: e(pubkey_g1, G2) * e(G1, -pubkey_g2) == 1

        BN254.G2Point memory neg_pubkey_g2 = negateG2(pubkey_g2);

        isValid = BN254.pairing(
            pubkey_g1,
            g2_gen,
            g1_gen,
            neg_pubkey_g2
        );
    }

    /**
     * @notice Safe version of validateKeyPair with gas limit
     * @param pubkey_g1 The public key on G1
     * @param pubkey_g2 The public key on G2
     * @param pairingGas Gas limit for the pairing check
     * @return success True if the pairing operation completed
     * @return isValid True if the keypair is valid
     */
    function safeValidateKeyPair(
        BN254.G1Point calldata pubkey_g1,
        BN254.G2Point calldata pubkey_g2,
        uint256 pairingGas
    ) external view returns (bool success, bool isValid) {
        BN254.G1Point memory g1_gen = BN254.generatorG1();
        BN254.G2Point memory g2_gen = BN254.generatorG2();
        BN254.G2Point memory neg_pubkey_g2 = negateG2(pubkey_g2);

        (success, isValid) = BN254.safePairing(
            pubkey_g1,
            g2_gen,
            g1_gen,
            neg_pubkey_g2,
            pairingGas
        );
    }

    /**
     * @notice Validates a keypair by checking against a known scalar multiplication
     * @param pubkey_g1 The public key on G1
     * @param pubkey_g2 The public key on G2
     * @param scalar A scalar to test with
     * @return isValid True if scalar * pubkey_g1 and scalar * pubkey_g2 satisfy the pairing relation
     *
     * @dev This is an alternative validation that also checks scalar multiplication consistency
     *      Useful for additional verification that the points are on the curve and properly formed
     */
    function validateKeyPairWithScalar(
        BN254.G1Point calldata pubkey_g1,
        BN254.G2Point calldata pubkey_g2,
        uint256 scalar
    ) external view returns (bool isValid) {
        // Compute scalar * pubkey_g1
        BN254.G1Point memory scaled_g1 = BN254.scalar_mul(pubkey_g1, scalar);

        // Check if e(scaled_g1, G2) == e(G1, scalar * pubkey_g2)
        // This validates both the keypair AND that scalar multiplication works correctly

        BN254.G2Point memory g2_gen = BN254.generatorG2();
        BN254.G1Point memory g1_gen = BN254.generatorG1();

        // We can't easily do G2 scalar mul in Solidity (no precompile)
        // So instead we check: e(scaled_g1, G2) == e(scalar * G1, pubkey_g2)
        BN254.G1Point memory scaled_g1_gen = BN254.scalar_mul(g1_gen, scalar);
        BN254.G2Point memory neg_pubkey_g2 = negateG2(pubkey_g2);

        isValid = BN254.pairing(
            scaled_g1,
            g2_gen,
            scaled_g1_gen,
            neg_pubkey_g2
        );
    }

    /**
     * @notice Batch validates multiple keypairs
     * @param pubkeys_g1 Array of G1 public keys
     * @param pubkeys_g2 Array of G2 public keys (must match length of pubkeys_g1)
     * @return results Array of validation results for each keypair
     */
    function batchValidateKeyPairs(
        BN254.G1Point[] calldata pubkeys_g1,
        BN254.G2Point[] calldata pubkeys_g2
    ) external view returns (bool[] memory results) {
        require(pubkeys_g1.length == pubkeys_g2.length, "Array length mismatch");

        results = new bool[](pubkeys_g1.length);
        BN254.G1Point memory g1_gen = BN254.generatorG1();
        BN254.G2Point memory g2_gen = BN254.generatorG2();

        for (uint256 i = 0; i < pubkeys_g1.length; i++) {
            BN254.G2Point memory neg_pubkey_g2 = negateG2(pubkeys_g2[i]);

            results[i] = BN254.pairing(
                pubkeys_g1[i],
                g2_gen,
                g1_gen,
                neg_pubkey_g2
            );
        }
    }

    /**
     * @notice Negates a G2 point
     * @param p The G2 point to negate
     * @return The negated G2 point
     * @dev G2 negation: -p = (x, -y) where negation is in the field Fp2
     */
    function negateG2(BN254.G2Point memory p) internal pure returns (BN254.G2Point memory) {
        // For G2 points on BN254, we negate by negating the y-coordinate
        // The y-coordinate is in Fp2, so we negate modulo FP_MODULUS
        uint256 q = BN254.FP_MODULUS;

        return BN254.G2Point({
            X: p.X, // X coordinate stays the same
            Y: [
                p.Y[0] == 0 ? 0 : q - (p.Y[0] % q),
                p.Y[1] == 0 ? 0 : q - (p.Y[1] % q)
            ]
        });
    }

    /**
     * @notice Verifies that a G1 point is the result of scalar multiplication
     * @param base Base point
     * @param scalar The scalar value
     * @param result Expected result of base * scalar
     * @return True if result == base * scalar
     */
    function verifyScalarMul(
        BN254.G1Point calldata base,
        uint256 scalar,
        BN254.G1Point calldata result
    ) external view returns (bool) {
        BN254.G1Point memory computed = BN254.scalar_mul(base, scalar);
        return (computed.X == result.X && computed.Y == result.Y);
    }
}
