//! 認証局（CA）証明書（F13）。
//!
//! HTTPS の中身を見るには、関所が「相手のサイトになりすました証明書」をその場で作り、
//! ブラウザに信頼してもらう必要がある。その証明書に署名するのが、ここで作る自分専用の認証局。
//! （hudsucker の `RcgenAuthority` が、接続のたびにサイトごとの証明書を発行してくれる）
//!
//! 仕組み:
//! 1. 初回起動時に認証局の証明書と秘密鍵を作り、アプリのデータフォルダに保存する
//! 2. ユーザーが証明書（公開してよい方）を macOS のキーチェーンに「常に信頼」で登録する。
//!    **アプリは勝手に登録しない**（手順を画面に出すだけ）
//! 3. 観測対象ホストへの HTTPS 接続だけ、関所が TLS をほどいて中を見る（TLS の終端）。
//!    ブラウザは関所が出した証明書を「信頼した認証局の署名がある」ので受け入れる
//!
//! 秘密鍵は PC の外に出さない。漏れると、この PC のブラウザに対して
//! どんなサイトにもなりすませてしまうため。ファイルは本人だけが読める権限（0600）で置く。
//! リポジトリにも入らない（保存先はアプリのデータフォルダで、プロジェクトの外）。

use std::fs;
use std::path::{Path, PathBuf};

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

/// 証明書（公開してよい）のファイル名。キーチェーンに登録するのはこちら
const CERT_FILE: &str = "api-lens-ca.pem";
/// 秘密鍵（絶対に外に出さない）のファイル名
const KEY_FILE: &str = "api-lens-ca.key";

/// 認証局のファイルの場所
#[derive(Debug, Clone)]
pub struct CaFiles {
    pub cert_path: PathBuf,
    pub key_path: PathBuf,
}

impl CaFiles {
    pub fn in_dir(dir: &Path) -> Self {
        Self {
            cert_path: dir.join(CERT_FILE),
            key_path: dir.join(KEY_FILE),
        }
    }

    /// 認証局がなければ作って保存する（初回起動時）。すでにあれば何もしない
    pub fn ensure(&self) -> Result<(), String> {
        if self.cert_path.exists() && self.key_path.exists() {
            return Ok(());
        }
        let dir = self
            .cert_path
            .parent()
            .ok_or("証明書の保存先が正しくありません")?;
        fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        restrict_permissions(dir, 0o700)?;

        let (cert, key) = generate_ca()?;
        // 鍵を先に書いて権限をしぼる（証明書だけあって鍵がない、という半端な状態を避ける順番）
        fs::write(&self.key_path, key).map_err(|e| e.to_string())?;
        restrict_permissions(&self.key_path, 0o600)?;
        fs::write(&self.cert_path, cert).map_err(|e| e.to_string())?;
        Ok(())
    }

    /// 保存してある認証局を読み込んで、hudsucker が使える形にする
    pub fn load(&self) -> Result<RcgenAuthority, String> {
        let cert = fs::read_to_string(&self.cert_path).map_err(|e| e.to_string())?;
        let key = fs::read_to_string(&self.key_path).map_err(|e| e.to_string())?;
        authority_from_pem(&cert, &key)
    }
}

/// ファイル・フォルダを本人だけが使える権限にする（macOS / Linux）
#[cfg(unix)]
fn restrict_permissions(path: &Path, mode: u32) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(mode)).map_err(|e| e.to_string())
}

/// Windows は全機能の完成後に確認する（N1）。いまは何もしない
#[cfg(not(unix))]
fn restrict_permissions(_path: &Path, _mode: u32) -> Result<(), String> {
    Ok(())
}

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

/// macOS で、この証明書が信頼済みとして登録されているかを調べる（目安）。
///
/// `security verify-cert` は、証明書が信頼できる認証局につながるときに成功する。
/// 自己署名の認証局なら「キーチェーンで信頼されているか」とほぼ同じ意味になる。
/// 調べられない環境では None。
pub fn is_trusted(cert_path: &Path) -> Option<bool> {
    if !cfg!(target_os = "macos") {
        return None;
    }
    let status = std::process::Command::new("/usr/bin/security")
        // -L: ネットワークに問い合わせない（ローカルだけで判定する）
        .args(["verify-cert", "-L", "-c"])
        .arg(cert_path)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .ok()?;
    Some(status.success())
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

    #[test]
    fn ensure_creates_once_and_keeps_key_private() {
        let dir =
            std::env::temp_dir().join(format!("api-lens-ca-test-{}", crate::models::now_ms()));
        let files = CaFiles::in_dir(&dir.join("ca"));

        files.ensure().unwrap();
        let first = fs::read_to_string(&files.cert_path).unwrap();
        // 2 回目は作り直さない（作り直すとキーチェーンの登録がやり直しになる）
        files.ensure().unwrap();
        assert_eq!(fs::read_to_string(&files.cert_path).unwrap(), first);
        files.load().unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = fs::metadata(&files.key_path).unwrap().permissions().mode();
            assert_eq!(mode & 0o777, 0o600);
        }
        fs::remove_dir_all(&dir).ok();
    }
}
