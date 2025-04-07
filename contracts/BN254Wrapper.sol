// SPDX-License-Identifier: MIT
pragma solidity ^0.8.27;

import "./BN254.sol";

/**
 * @title Wrapper for the BN254 elliptic curve operations library
 * @author Based on Layr Labs, Inc. BN254 library
 * @notice This contract provides callable functions that wrap the internal functions of the BN254 library
 */
contract BN254Wrapper {
    /**
     * @notice Get the generator point of G1
     * @return The generator point of G1
     */
    function getGeneratorG1() external pure returns (BN254.G1Point memory) {
        return BN254.generatorG1();
    }

    /**
     * @notice Get the generator point of G2
     * @return The generator point of G2
     */
    function getGeneratorG2() external pure returns (BN254.G2Point memory) {
        return BN254.generatorG2();
    }

    /**
     * @notice Get the negation of the generator point of G2
     * @return The negation of the generator point of G2
     */
    function getNegGeneratorG2() external pure returns (BN254.G2Point memory) {
        return BN254.negGeneratorG2();
    }

    /**
     * @notice Negate a point in G1
     * @param p The point to negate
     * @return The negation of the input point
     */
    function negate(BN254.G1Point calldata p) external pure returns (BN254.G1Point memory) {
        return BN254.negate(p);
    }

    /**
     * @notice Add two points in G1
     * @param p1 First point
     * @param p2 Second point
     * @return The sum of the two points
     */
    function plus(BN254.G1Point calldata p1, BN254.G1Point calldata p2) external view returns (BN254.G1Point memory) {
        return BN254.plus(p1, p2);
    }

    /**
     * @notice Multiply a point in G1 by a small scalar
     * @param p The point to multiply
     * @param s The scalar to multiply by (must be less than 2^9)
     * @return The product of the point and scalar
     */
    function scalar_mul_tiny(BN254.G1Point calldata p, uint16 s) external view returns (BN254.G1Point memory) {
        return BN254.scalar_mul_tiny(p, s);
    }

    /**
     * @notice Multiply a point in G1 by a scalar
     * @param p The point to multiply
     * @param s The scalar to multiply by
     * @return The product of the point and scalar
     */
    function scalar_mul(BN254.G1Point calldata p, uint256 s) external view returns (BN254.G1Point memory) {
        return BN254.scalar_mul(p, s);
    }

    /**
     * @notice Perform a pairing check
     * @param a1 First G1 point
     * @param a2 First G2 point
     * @param b1 Second G1 point
     * @param b2 Second G2 point
     * @return True if the pairing check passes
     */
    function pairing(
        BN254.G1Point calldata a1,
        BN254.G2Point calldata a2,
        BN254.G1Point calldata b1,
        BN254.G2Point calldata b2
    ) external view returns (bool) {
        return BN254.pairing(a1, a2, b1, b2);
    }

    /**
     * @notice Perform a pairing check with a gas limit
     * @param a1 First G1 point
     * @param a2 First G2 point
     * @param b1 Second G1 point
     * @param b2 Second G2 point
     * @param pairingGas Gas limit for the pairing check
     * @return success True if the operation completed successfully
     * @return result True if the pairing check passes
     */
    function safePairing(
        BN254.G1Point calldata a1,
        BN254.G2Point calldata a2,
        BN254.G1Point calldata b1,
        BN254.G2Point calldata b2,
        uint256 pairingGas
    ) external view returns (bool success, bool result) {
        return BN254.safePairing(a1, a2, b1, b2, pairingGas);
    }

    /**
     * @notice Hash a G1 point
     * @param pk The G1 point to hash
     * @return The keccak256 hash of the G1 point
     */
    function hashG1Point(BN254.G1Point calldata pk) external pure returns (bytes32) {
        return BN254.hashG1Point(pk);
    }

    /**
     * @notice Hash a G2 point
     * @param pk The G2 point to hash
     * @return The keccak256 hash of the G2 point
     */
    function hashG2Point(BN254.G2Point calldata pk) external pure returns (bytes32) {
        return BN254.hashG2Point(pk);
    }

    /**
     * @notice Map a bytes32 value to a G1 point
     * @param _x The bytes32 value to map
     * @return The G1 point corresponding to the input value
     */
    function hashToG1(bytes32 _x) external view returns (BN254.G1Point memory) {
        return BN254.hashToG1(_x);
    }

    /**
     * @notice Find a y-coordinate for a given x-coordinate on the curve
     * @param x The x-coordinate
     * @return beta x^3 + b
     * @return y The corresponding y-coordinate
     */
    function findYFromX(uint256 x) external view returns (uint256 beta, uint256 y) {
        return BN254.findYFromX(x);
    }

    /**
     * @notice Perform modular exponentiation
     * @param _base The base
     * @param _exponent The exponent
     * @param _modulus The modulus
     * @return The result of (_base^_exponent) % _modulus
     */
    function expMod(uint256 _base, uint256 _exponent, uint256 _modulus) external view returns (uint256) {
        return BN254.expMod(_base, _exponent, _modulus);
    }

    /**
     * @notice Get the FP_MODULUS constant
     * @return The FP_MODULUS constant
     */
    function getFPModulus() external pure returns (uint256) {
        return BN254.FP_MODULUS;
    }

    /**
     * @notice Get the FR_MODULUS constant
     * @return The FR_MODULUS constant
     */
    function getFRModulus() external pure returns (uint256) {
        return BN254.FR_MODULUS;
    }
}
