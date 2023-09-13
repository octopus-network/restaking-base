use near_sdk::env;
use regex::Regex;

pub fn assert_attached_near() {
    assert!(env::attached_deposit() > 0, "No near attached.")
}

pub mod u64_dec_format {
    use near_sdk::serde::de;
    use near_sdk::serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(num: &u64, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&num.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<u64, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(de::Error::custom)
    }
}

pub mod u128_dec_format {
    use near_sdk::serde::de;
    use near_sdk::serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(num: &u128, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&num.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<u128, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(de::Error::custom)
    }
}

// validate rule refer to https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-2.md
pub fn validate_chain_id(chain_id: &String) {
    let chain_id_regex = Regex::new(r"^[-a-z0-9]{3,8}:[-_a-zA-Z0-9]{1,32}$").unwrap();
    assert!(
        chain_id_regex.is_match(chain_id),
        "Failed to validate_chain_id({}).",
        chain_id
    );
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    // #[should_panic(expect="")]
    fn test_validate_correct_chain_id() {
        // let mut context = VMContextBuilder::new();
        validate_chain_id(&"eip155:1".to_string());
        validate_chain_id(&"bip122:000000000019d6689c085ae165831e93".to_string());
        validate_chain_id(&"bip122:12a765e31ffd4059bada1e25190f6e98".to_string());
        validate_chain_id(&"bip122:fdbe99b90c90bae7505796461471d89a".to_string());
        validate_chain_id(&"cosmos:cosmoshub-2".to_string());
        validate_chain_id(&"cosmos:cosmoshub-3".to_string());
        validate_chain_id(&"cosmos:Binance-Chain-Tigris".to_string());
        validate_chain_id(&"cosmos:iov-mainnet".to_string());
        validate_chain_id(&"starknet:SN_GOERLI".to_string());
        validate_chain_id(&"lip9:9ee11e9df416b18b".to_string());
        validate_chain_id(&"chainstd:8c3444cf8970a9e41a706fab93e7a6c4".to_string());

        // testing_env!(context.block_timestamp(1 * 1000_000_000).build());
    }

    #[test]
    #[should_panic]
    fn test_validate_uncorrect_chain_id_1() {
        validate_chain_id(&"ab:reference12345".to_string());
    }

    #[test]
    #[should_panic]
    fn test_validate_uncorrect_chain_id_2() {
        validate_chain_id(&"abcdefghi:reference12345".to_string());
    }

    #[test]
    #[should_panic]
    fn test_validate_uncorrect_chain_id_3() {
        validate_chain_id(&"n@mespace:reference1234".to_string());
    }

    #[test]
    #[should_panic]
    fn test_validate_uncorrect_chain_id_4() {
        validate_chain_id(&"namespace:r".to_string());
    }

    #[test]
    #[should_panic]
    fn test_validate_uncorrect_chain_id_5() {
        validate_chain_id(&"namespace:abcdefghijklmnopqrstuvwxyabcdefghijklmnop".to_string());
    }

    #[test]
    #[should_panic]
    fn test_validate_uncorrect_chain_id_6() {
        validate_chain_id(&"namespace:ref#erence12345".to_string());
    }

    #[test]
    #[should_panic]
    fn test_validate_uncorrect_chain_id_7() {
        validate_chain_id(&"namespacereference12345".to_string());
    }
}
