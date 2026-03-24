
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum NssfFeature {
    Nssrg = 0,
    Nsag = 1,
    EnhancedRoaming = 2,
    SlicePriority = 3,
    DynamicMapping = 4,
}

const NSSF_SUPPORTED_FEATURES: &[(NssfFeature, bool)] = &[
    (NssfFeature::Nssrg, false),
    (NssfFeature::Nsag, false),
    (NssfFeature::EnhancedRoaming, true),
    (NssfFeature::SlicePriority, true),
    (NssfFeature::DynamicMapping, true),
];

fn is_feature_supported_by_nssf(bit_index: usize) -> bool {
    NSSF_SUPPORTED_FEATURES
        .iter()
        .find(|(f, _)| *f as usize == bit_index)
        .map(|(_, supported)| *supported)
        .unwrap_or(false)
}

fn hex_char_to_bits(c: u8) -> [bool; 4] {
    let value = match c {
        b'0'..=b'9' => c - b'0',
        b'a'..=b'f' => c - b'a' + 10,
        _ => 0,
    };
    [
        (value & 0x1) != 0,
        (value & 0x2) != 0,
        (value & 0x4) != 0,
        (value & 0x8) != 0,
    ]
}

fn bits_to_hex_char(bits: &[bool]) -> u8 {
    let mut value: u8 = 0;
    if bits.first().copied().unwrap_or(false) {
        value |= 0x1;
    }
    if bits.get(1).copied().unwrap_or(false) {
        value |= 0x2;
    }
    if bits.get(2).copied().unwrap_or(false) {
        value |= 0x4;
    }
    if bits.get(3).copied().unwrap_or(false) {
        value |= 0x8;
    }
    if value < 10 {
        b'0' + value
    } else {
        b'a' + (value - 10)
    }
}

fn parse_feature_string(features: &str) -> Vec<bool> {
    let bytes = features.as_bytes();
    let mut bits = Vec::new();

    for &b in bytes.iter().rev() {
        let hex_bits = hex_char_to_bits(b);
        bits.extend_from_slice(&hex_bits);
    }

    bits
}

fn create_feature_string(bits: &[bool]) -> Option<String> {
    if bits.is_empty() {
        return None;
    }

    let mut result = Vec::new();

    for chunk in bits.chunks(4) {
        result.push(bits_to_hex_char(chunk));
    }

    while result.len() > 1 && result.last() == Some(&b'0') {
        result.pop();
    }

    result.reverse();
    Some(String::from_utf8(result).unwrap_or_default())
}

pub fn negotiate_features(consumer_features: Option<&str>) -> Option<String> {
    let consumer = consumer_features?;

    if consumer.is_empty() {
        return None;
    }

    if !consumer.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }

    let consumer_bits = parse_feature_string(&consumer.to_ascii_lowercase());

    let mut negotiated_bits: Vec<bool> = Vec::new();

    for (i, &bit) in consumer_bits.iter().enumerate() {
        negotiated_bits.push(bit && is_feature_supported_by_nssf(i));
    }

    while negotiated_bits.last() == Some(&false) {
        negotiated_bits.pop();
    }

    if negotiated_bits.is_empty() {
        return None;
    }

    create_feature_string(&negotiated_bits)
}

pub fn is_feature_negotiated(negotiated_features: Option<&str>, feature: NssfFeature) -> bool {
    let Some(features) = negotiated_features else {
        return false;
    };

    let bits = parse_feature_string(&features.to_ascii_lowercase());
    let index = feature as usize;

    bits.get(index).copied().unwrap_or(false)
}

pub fn validate_required_features(
    negotiated_features: Option<&str>,
    required_features: Option<&str>,
) -> bool {
    let Some(required) = required_features else {
        return true;
    };

    let Some(negotiated) = negotiated_features else {
        return false;
    };

    let negotiated_bits = parse_feature_string(&negotiated.to_ascii_lowercase());
    let required_bits = parse_feature_string(&required.to_ascii_lowercase());

    for (i, &required_bit) in required_bits.iter().enumerate() {
        if required_bit && !negotiated_bits.get(i).copied().unwrap_or(false) {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_negotiate_all_features_requested() {
        assert_eq!(negotiate_features(Some("1f")), Some("1c".to_string()));
    }

    #[test]
    fn test_negotiate_lower_features_requested() {
        assert_eq!(negotiate_features(Some("7")), Some("4".to_string()));
    }

    #[test]
    fn test_negotiate_enhanced_roaming_only() {
        assert_eq!(negotiate_features(Some("4")), Some("4".to_string()));
    }

    #[test]
    fn test_negotiate_slice_priority_only() {
        assert_eq!(negotiate_features(Some("8")), Some("8".to_string()));
    }

    #[test]
    fn test_negotiate_dynamic_mapping_only() {
        assert_eq!(negotiate_features(Some("10")), Some("10".to_string()));
    }

    #[test]
    fn test_negotiate_nssrg_not_supported() {
        assert_eq!(negotiate_features(Some("1")), None);
    }

    #[test]
    fn test_negotiate_nsag_not_supported() {
        assert_eq!(negotiate_features(Some("2")), None);
    }

    #[test]
    fn test_negotiate_none_input() {
        assert_eq!(negotiate_features(None), None);
    }

    #[test]
    fn test_negotiate_empty_input() {
        assert_eq!(negotiate_features(Some("")), None);
    }

    #[test]
    fn test_negotiate_invalid_hex() {
        assert_eq!(negotiate_features(Some("xyz")), None);
    }

    #[test]
    fn test_negotiate_case_insensitive() {
        assert_eq!(negotiate_features(Some("1F")), Some("1c".to_string()));
    }

    #[test]
    fn test_is_feature_negotiated() {
        assert!(is_feature_negotiated(Some("1c"), NssfFeature::EnhancedRoaming));
        assert!(is_feature_negotiated(Some("1c"), NssfFeature::SlicePriority));
        assert!(is_feature_negotiated(Some("1c"), NssfFeature::DynamicMapping));
        assert!(!is_feature_negotiated(Some("1c"), NssfFeature::Nssrg));
        assert!(!is_feature_negotiated(Some("1c"), NssfFeature::Nsag));
    }

    #[test]
    fn test_is_feature_negotiated_none() {
        assert!(!is_feature_negotiated(None, NssfFeature::EnhancedRoaming));
    }

    #[test]
    fn test_validate_required_features_subset() {
        assert!(validate_required_features(Some("1c"), Some("4")));
        assert!(validate_required_features(Some("1c"), Some("c")));
    }

    #[test]
    fn test_validate_required_features_not_met() {
        assert!(!validate_required_features(Some("4"), Some("1c")));
    }

    #[test]
    fn test_validate_required_features_no_required() {
        assert!(validate_required_features(Some("1c"), None));
    }

    #[test]
    fn test_validate_required_features_no_negotiated() {
        assert!(!validate_required_features(None, Some("4")));
    }

    #[test]
    fn test_parse_and_create_roundtrip() {
        let bits = parse_feature_string("1c");
        let result = create_feature_string(&bits);
        assert_eq!(result, Some("1c".to_string()));
    }
}
