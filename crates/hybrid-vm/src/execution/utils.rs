/// Utility functions for the Hybrid VM execution module.
use reth::revm::primitives::Address;

pub fn __3u64_to_address(limb_one: u64, limb_two: u64, limb_three: u64) -> Address {
    let mut bytes = [0u8; 20];
    bytes[0..8].copy_from_slice(&limb_one.to_be_bytes());
    bytes[8..16].copy_from_slice(&limb_two.to_be_bytes());
    bytes[16..20].copy_from_slice(&limb_three.to_be_bytes()[4..]);
    Address::from_slice(&bytes)
}

pub fn __address_to_3u64(address: Address) -> (u64, u64, u64) {
    let bytes = address.0;
    let limb_one = u64::from_be_bytes(bytes[0..8].try_into().unwrap());
    let limb_two = u64::from_be_bytes(bytes[8..16].try_into().unwrap());
    let mut buf = [0u8; 8];
    buf[4..].copy_from_slice(&bytes[16..20]);
    let limb_three = u64::from_be_bytes(buf);
    (limb_one, limb_two, limb_three)
}

#[cfg(test)]
mod tests {
    use super::*;
    use reth::revm::primitives::Address;

    #[test]
    fn test_3u64_to_address_zero() {
        let address = __3u64_to_address(0, 0, 0);
        assert_eq!(address, Address::ZERO);
    }

    #[test]
    fn test_3u64_to_address_known_values() {
        // Test with known non-zero values
        let limb1 = 0x0123456789abcdef;
        let limb2 = 0xfedcba9876543210;
        let limb3 = 0x12345678; // Only first 4 bytes will be used

        let address = __3u64_to_address(limb1, limb2, limb3);

        let expected_bytes = [
            0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, // limb1
            0xfe, 0xdc, 0xba, 0x98, 0x76, 0x54, 0x32, 0x10, // limb2
            0x12, 0x34, 0x56, 0x78, // first 4 bytes of limb3
        ];

        assert_eq!(address, Address::new(expected_bytes));
    }

    #[test]
    fn test_address_to_3u64_zero() {
        let (limb1, limb2, limb3) = __address_to_3u64(Address::ZERO);
        assert_eq!(limb1, 0);
        assert_eq!(limb2, 0);
        assert_eq!(limb3, 0);
    }

    #[test]
    fn test_address_to_3u64_known_values() {
        let bytes = [
            0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, // expected limb1
            0xfe, 0xdc, 0xba, 0x98, 0x76, 0x54, 0x32, 0x10, // expected limb2
            0x12, 0x34, 0x56, 0x78, // expected limb3 (padded)
        ];
        let address = Address::from_slice(&bytes);

        let (limb1, limb2, limb3) = __address_to_3u64(address);

        assert_eq!(limb1, 0x0123456789abcdef);
        assert_eq!(limb2, 0xfedcba9876543210);
        assert_eq!(limb3, 0x12345678); // Should be padded with zeros in high bytes
    }

    #[test]
    fn test_round_trip_conversion_zero() {
        // Test zero address round trip
        let original = Address::ZERO;
        let (l1, l2, l3) = __address_to_3u64(original);
        let converted = __3u64_to_address(l1, l2, l3);
        assert_eq!(original, converted);
    }
}
