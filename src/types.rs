use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use revm::primitives::{Address, U256};
use std::str::FromStr;

pub fn parse_address(addr: &str) -> PyResult<Address> {
    let addr = addr.trim_start_matches("0x");
    Address::from_str(addr)
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
    fn test_u256_from_u128() {
        let val: u128 = 10u128.pow(18);
        assert_eq!(py_int_to_u256(val), U256::from(val));
    }
}
