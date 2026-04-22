//! Round-trip and failure-mode tests for [`AgeArchiver`].

use wardnet_common::backup::BundleManifest;
use wardnetd_data::secret_store::SecretEntry;

use crate::backup::archiver::{AgeArchiver, BackupArchiver, BundleContents};

fn sample_contents() -> BundleContents {
    BundleContents {
        manifest: BundleManifest::new("0.2.0-test", 7, "test-host", 2),
        database_bytes: b"SQLite format 3\x00fake-db-bytes".to_vec(),
        config_bytes: b"[database]\npath = \"wardnet.db\"\n".to_vec(),
        secrets: vec![
            SecretEntry {
                path: "wireguard/aaa.key".to_owned(),
                value: b"priv-key-aaa".to_vec(),
            },
            SecretEntry {
                path: "wireguard/bbb.key".to_owned(),
                value: b"priv-key-bbb".to_vec(),
            },
        ],
    }
}

#[tokio::test]
async fn pack_unpack_round_trip() {
    let archiver = AgeArchiver::new();
    let contents = sample_contents();

    let encrypted = archiver
        .pack("correct-horse-battery-staple", contents.clone())
        .await
        .unwrap();

    let decoded = archiver
        .unpack("correct-horse-battery-staple", &encrypted)
        .await
        .unwrap();

    assert_eq!(decoded.manifest, contents.manifest);
    assert_eq!(decoded.database_bytes, contents.database_bytes);
    assert_eq!(decoded.config_bytes, contents.config_bytes);

    let mut got: Vec<_> = decoded.secrets.into_iter().collect();
    got.sort_by(|a, b| a.path.cmp(&b.path));
    let mut want: Vec<_> = contents.secrets.into_iter().collect();
    want.sort_by(|a, b| a.path.cmp(&b.path));
    assert_eq!(got.len(), want.len());
    for (g, w) in got.iter().zip(want.iter()) {
        assert_eq!(g.path, w.path);
        assert_eq!(g.value, w.value);
    }
}

#[tokio::test]
async fn unpack_fails_with_wrong_passphrase() {
    let archiver = AgeArchiver::new();
    let encrypted = archiver
        .pack("real-passphrase-1234", sample_contents())
        .await
        .unwrap();

    let err = archiver
        .unpack("wrong-passphrase-5678", &encrypted)
        .await
        .unwrap_err();
    assert!(
        format!("{err:#}").to_lowercase().contains("decryption"),
        "expected decryption error, got: {err}"
    );
}

#[tokio::test]
async fn unpack_rejects_garbage_bytes() {
    let archiver = AgeArchiver::new();
    let err = archiver
        .unpack("whatever", b"not-an-age-stream")
        .await
        .unwrap_err();
    assert!(
        format!("{err:#}").to_lowercase().contains("age"),
        "expected age-layer error, got: {err}"
    );
}

#[tokio::test]
async fn pack_unpack_round_trip_with_empty_secrets_list() {
    let archiver = AgeArchiver::new();
    let contents = BundleContents {
        manifest: BundleManifest::new("0.2.0-test", 7, "no-secrets-host", 0),
        database_bytes: b"db".to_vec(),
        config_bytes: b"cfg".to_vec(),
        secrets: Vec::new(),
    };

    let encrypted = archiver
        .pack("some-passphrase-that-is-long-enough", contents.clone())
        .await
        .unwrap();

    let decoded = archiver
        .unpack("some-passphrase-that-is-long-enough", &encrypted)
        .await
        .unwrap();

    assert_eq!(decoded.manifest, contents.manifest);
    assert_eq!(decoded.database_bytes, contents.database_bytes);
    assert_eq!(decoded.config_bytes, contents.config_bytes);
    assert!(decoded.secrets.is_empty());
}

#[test]
fn bundle_contents_debug_redacts_payload_bytes() {
    let contents = BundleContents {
        manifest: BundleManifest::new("0.2.0-test", 7, "redact-host", 1),
        database_bytes: b"PRAGMA foreign_keys=ON; -- secret dump bytes".to_vec(),
        config_bytes: b"[admin]\npassword = \"secret-leaked\"".to_vec(),
        secrets: vec![SecretEntry {
            path: "wireguard/k.key".to_owned(),
            value: b"priv-key-plaintext".to_vec(),
        }],
    };
    let rendered = format!("{contents:?}");
    assert!(rendered.contains("redact-host"));
    assert!(rendered.contains("bytes"));
    assert!(rendered.contains("1 entries"));
    assert!(!rendered.contains("PRAGMA"));
    assert!(!rendered.contains("secret-leaked"));
    assert!(!rendered.contains("priv-key-plaintext"));
}

#[tokio::test]
async fn unpack_rejects_bundle_with_no_manifest() {
    use std::io::Write;

    use age::secrecy::SecretString;
    use flate2::Compression;
    use flate2::write::GzEncoder;

    // Craft a tar.gz with only wardnet.db + wardnet.toml (no manifest.json),
    // encrypt it with age, and hand it to the archiver. The unpack path
    // should reject it as "missing manifest.json".
    let mut compressed: Vec<u8> = Vec::new();
    {
        let gz = GzEncoder::new(&mut compressed, Compression::default());
        let mut tar = tar::Builder::new(gz);
        tar.mode(tar::HeaderMode::Deterministic);

        for (path, payload) in [("wardnet.db", b"db" as &[u8]), ("wardnet.toml", b"cfg")] {
            let mut header = tar::Header::new_gnu();
            header.set_size(payload.len() as u64);
            header.set_mode(0o600);
            header.set_mtime(0);
            header.set_cksum();
            tar.append_data(&mut header, path, payload).unwrap();
        }
        tar.finish().unwrap();
        let gz = tar.into_inner().unwrap();
        gz.finish().unwrap();
    }

    let passphrase = SecretString::from("correct-horse-battery-staple".to_owned());
    let encryptor = age::Encryptor::with_user_passphrase(passphrase);
    let mut encrypted: Vec<u8> = Vec::new();
    let mut writer = encryptor.wrap_output(&mut encrypted).unwrap();
    writer.write_all(&compressed).unwrap();
    writer.finish().unwrap();

    let archiver = AgeArchiver::new();
    let err = archiver
        .unpack("correct-horse-battery-staple", &encrypted)
        .await
        .unwrap_err();
    assert!(
        format!("{err:#}").to_lowercase().contains("manifest"),
        "expected missing-manifest error, got: {err}"
    );
}

#[tokio::test]
async fn pack_is_non_empty_and_not_plaintext() {
    let archiver = AgeArchiver::new();
    let contents = sample_contents();
    let plaintext_marker = &contents.secrets[0].value.clone();

    let encrypted = archiver
        .pack("a-reasonable-passphrase", contents)
        .await
        .unwrap();

    assert!(!encrypted.is_empty());
    // The plaintext marker must not appear in the ciphertext.
    assert!(
        !encrypted
            .windows(plaintext_marker.len())
            .any(|w| w == plaintext_marker.as_slice()),
        "encrypted output should not contain plaintext secret bytes"
    );
}
