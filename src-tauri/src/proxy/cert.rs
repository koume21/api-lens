//! 認証局（CA）証明書。
//!
//! HTTPS の中身を見るには、関所が「相手のサイトになりすました証明書」をその場で作り、
//! ブラウザに信頼してもらう必要がある。その証明書に署名するのが、ここで作る自分専用の認証局。
//! （hudsucker の `RcgenAuthority` が、接続のたびにサイトごとの証明書を発行してくれる）

use hudsucker::certificate_authority::RcgenAuthority;
use hudsucker::rcgen::{
    BasicConstraints, CertificateParams, DistinguishedName, DnType, IsCa, Issuer, KeyPair,
    KeyUsagePurpose,
};
use hudsucker::rustls::crypto::aws_lc_rs;

/// 発行したサイト証明書を何件までメモリに覚えておくか（同じサイトに毎回作り直さないため）
const CERT_CACHE_SIZE: u64 = 1_000;

/// 証明書に書く名前。キーチェーンアクセスで探すときにこの名前で見つかる
pub const CA_COMMON_NAME: &str = "api-lens Local CA";

/// 新しい認証局の証明書と秘密鍵を作る（PEM 形式の文字列で返す）
pub fn generate_ca() -> Result<(String, String), String> {
    let key = KeyPair::generate().map_err(|e| e.to_string())?;

    let mut params = CertificateParams::default();
    let mut dn = DistinguishedName::new();
    dn.push(DnType::CommonName, CA_COMMON_NAME);
    dn.push(DnType::OrganizationName, "api-lens");
    params.distinguished_name = dn;
    // 「ほかの証明書に署名してよい」認証局であることを示す
    params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    params.key_usages = vec![
        KeyUsagePurpose::KeyCertSign,
        KeyUsagePurpose::CrlSign,
        KeyUsagePurpose::DigitalSignature,
    ];

    let cert = params.self_signed(&key).map_err(|e| e.to_string())?;
    Ok((cert.pem(), key.serialize_pem()))
}

/// PEM の証明書と秘密鍵から、hudsucker が使う認証局を作る
pub fn authority_from_pem(cert_pem: &str, key_pem: &str) -> Result<RcgenAuthority, String> {
    let key = KeyPair::from_pem(key_pem).map_err(|e| e.to_string())?;
    let issuer = Issuer::from_ca_cert_pem(cert_pem, key).map_err(|e| e.to_string())?;
    Ok(RcgenAuthority::new(
        issuer,
        CERT_CACHE_SIZE,
        aws_lc_rs::default_provider(),
    ))
}

/// メモリ上だけで使う使い捨ての認証局（HTTPS を開かないときの穴埋め）
pub fn ephemeral_authority() -> Result<RcgenAuthority, String> {
    let (cert, key) = generate_ca()?;
    authority_from_pem(&cert, &key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_ca_can_be_loaded() {
        let (cert, key) = generate_ca().unwrap();
        assert!(cert.starts_with("-----BEGIN CERTIFICATE-----"));
        assert!(key.contains("PRIVATE KEY"));
        authority_from_pem(&cert, &key).unwrap();
    }
}
