use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use revm::primitives::{Address, U256};
use std::str::FromStr;

pub fn parse_address(addr: &str) -> PyResult<Address> {
    let stripped = addr.trim_start_matches("0x");
    Address::from_str(stripped)
        .map_err(|e| PyValueError::new_err(format!("Invalid address '{}': {}", addr, e)))
}

pub fn address_to_hex(addr: Address) -> String {
    format!("0x{addr:x}")
}

pub fn py_int_to_u256(val: u128) -> U256 {
    U256::from(val)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_address() {
        assert!(parse_address("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045").is_ok());
    }

    #[test]
    fn test_parse_without_prefix() {
        assert!(parse_address("d8dA6BF26964aF9D7eEd9e03E53415D37aA96045").is_ok());
    }

    #[test]
    fn test_parse_invalid() {
        assert!(parse_address("not_an_address").is_err());
    }

    #[test]
    fn test_u256_one_eth() {
        // 1 ETH in wei = 10^18 = 0xde0b6b3a7640000
        let one_eth = py_int_to_u256(1_000_000_000_000_000_000u128);
        assert_eq!(one_eth, U256::from_str_radix("de0b6b3a7640000", 16).unwrap());
    }

    #[test]
    fn test_address_to_hex_roundtrip() {
        let input = "d8da6bf26964af9d7eed9e03e53415d37aa96045";
        let addr = parse_address(input).unwrap();
        assert_eq!(address_to_hex(addr), format!("0x{input}"));
    }
}
