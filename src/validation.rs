use crate::errors::{AppError, AppResult};
use crate::types::common::{PlmnId, Snssai, Tai};

pub fn validate_plmn(plmn: &PlmnId) -> bool {
    let mcc = &plmn.mcc;
    let mnc = &plmn.mnc;

    if mcc.len() != 3 || !mcc.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }

    if !(2..=3).contains(&mnc.len()) || !mnc.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }

    true
}

pub fn validate_snssai(snssai: &Snssai) -> bool {
    if let Some(ref sd) = snssai.sd {
        if sd.len() != 6 || !sd.chars().all(|c| c.is_ascii_hexdigit()) {
            return false;
        }
    }
    true
}

pub fn validate_tai(tai: &Tai) -> bool {
    if !validate_plmn(&tai.plmn_id) {
        return false;
    }

    let tac = &tai.tac;
    if tac.is_empty() || tac.len() > 6 {
        return false;
    }
    if !tac.chars().all(|c| c.is_ascii_hexdigit()) {
        return false;
    }

    true
}

pub fn validate_supi(supi: &str) -> Option<(String, PlmnId)> {
    let imsi = supi.strip_prefix("imsi-")?;

    if imsi.len() < 10 || imsi.len() > 15 {
        return None;
    }

    if !imsi.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }

    let mcc = imsi[..3].to_string();
    let mnc = if imsi.len() >= 6 {
        let mnc3 = &imsi[3..6];
        let mnc2 = &imsi[3..5];
        if is_two_digit_mnc(&mcc) {
            mnc2.to_string()
        } else {
            mnc3.to_string()
        }
    } else {
        return None;
    };

    let plmn = PlmnId { mcc, mnc };
    Some((imsi.to_string(), plmn))
}

fn is_two_digit_mnc(mcc: &str) -> bool {
    matches!(
        mcc,
        "302" | "310" | "311" | "312" | "313" | "314" | "315" | "316"
    )
}

pub fn validate_snssai_list(list: &[Snssai]) -> AppResult<()> {
    if list.len() > 8 {
        return Err(AppError::BadRequest(
            "S-NSSAI list exceeds maximum of 8 entries".to_string(),
        ));
    }

    for (i, snssai) in list.iter().enumerate() {
        if !validate_snssai(snssai) {
            return Err(AppError::BadRequest(format!(
                "Invalid S-NSSAI at index {}",
                i
            )));
        }
    }

    for i in 0..list.len() {
        for j in (i + 1)..list.len() {
            if snssai_eq(&list[i], &list[j]) {
                return Err(AppError::BadRequest(format!(
                    "Duplicate S-NSSAI at indices {} and {}",
                    i, j
                )));
            }
        }
    }

    Ok(())
}

pub fn snssai_eq(a: &Snssai, b: &Snssai) -> bool {
    if a.sst != b.sst {
        return false;
    }

    match (&a.sd, &b.sd) {
        (None, None) => true,
        (Some(sd_a), Some(sd_b)) => normalize_sd(sd_a) == normalize_sd(sd_b),
        (Some(sd), None) | (None, Some(sd)) => normalize_sd(sd) == "000000",
    }
}

fn normalize_sd(sd: &str) -> String {
    sd.to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_plmn_valid() {
        assert!(validate_plmn(&PlmnId {
            mcc: "001".to_string(),
            mnc: "01".to_string(),
        }));
        assert!(validate_plmn(&PlmnId {
            mcc: "310".to_string(),
            mnc: "410".to_string(),
        }));
    }

    #[test]
    fn test_validate_plmn_invalid() {
        assert!(!validate_plmn(&PlmnId {
            mcc: "AB1".to_string(),
            mnc: "01".to_string(),
        }));
        assert!(!validate_plmn(&PlmnId {
            mcc: "01".to_string(),
            mnc: "01".to_string(),
        }));
        assert!(!validate_plmn(&PlmnId {
            mcc: "001".to_string(),
            mnc: "1".to_string(),
        }));
    }

    #[test]
    fn test_validate_snssai_valid() {
        assert!(validate_snssai(&Snssai { sst: 1, sd: None }));
        assert!(validate_snssai(&Snssai {
            sst: 1,
            sd: Some("000001".to_string()),
        }));
        assert!(validate_snssai(&Snssai {
            sst: 255,
            sd: Some("ABCDEF".to_string()),
        }));
    }

    #[test]
    fn test_validate_snssai_invalid_sd() {
        assert!(!validate_snssai(&Snssai {
            sst: 1,
            sd: Some("00001".to_string()),
        }));
        assert!(!validate_snssai(&Snssai {
            sst: 1,
            sd: Some("GGGGGG".to_string()),
        }));
    }

    #[test]
    fn test_validate_supi() {
        let result = validate_supi("imsi-001010000000001");
        assert!(result.is_some());
        let (imsi, plmn) = result.unwrap();
        assert_eq!(imsi, "001010000000001");
        assert_eq!(plmn.mcc, "001");
        assert_eq!(plmn.mnc, "010");
    }

    #[test]
    fn test_validate_supi_invalid() {
        assert!(validate_supi("001010000000001").is_none());
        assert!(validate_supi("imsi-12345").is_none());
        assert!(validate_supi("imsi-ABCDE0000000001").is_none());
    }

    #[test]
    fn test_snssai_eq_basic() {
        let a = Snssai { sst: 1, sd: None };
        let b = Snssai { sst: 1, sd: None };
        assert!(snssai_eq(&a, &b));
    }

    #[test]
    fn test_snssai_eq_sd_normalization() {
        let a = Snssai {
            sst: 1,
            sd: Some("abcdef".to_string()),
        };
        let b = Snssai {
            sst: 1,
            sd: Some("ABCDEF".to_string()),
        };
        assert!(snssai_eq(&a, &b));
    }

    #[test]
    fn test_snssai_eq_none_vs_zeros() {
        let a = Snssai { sst: 1, sd: None };
        let b = Snssai {
            sst: 1,
            sd: Some("000000".to_string()),
        };
        assert!(snssai_eq(&a, &b));
    }

    #[test]
    fn test_snssai_eq_different() {
        let a = Snssai { sst: 1, sd: None };
        let b = Snssai { sst: 2, sd: None };
        assert!(!snssai_eq(&a, &b));
    }

    #[test]
    fn test_validate_snssai_list_ok() {
        let list = vec![
            Snssai { sst: 1, sd: None },
            Snssai { sst: 2, sd: None },
        ];
        assert!(validate_snssai_list(&list).is_ok());
    }

    #[test]
    fn test_validate_snssai_list_too_many() {
        let list: Vec<Snssai> = (0..9).map(|i| Snssai { sst: i, sd: None }).collect();
        assert!(validate_snssai_list(&list).is_err());
    }

    #[test]
    fn test_validate_snssai_list_duplicates() {
        let list = vec![
            Snssai { sst: 1, sd: None },
            Snssai {
                sst: 1,
                sd: Some("000000".to_string()),
            },
        ];
        assert!(validate_snssai_list(&list).is_err());
    }
}
