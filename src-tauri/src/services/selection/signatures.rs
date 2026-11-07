//! Verificación Ed25519 de catálogos y manifiestos de TFL Selection.
//!
//! La clave pública de confianza viaja DENTRO del launcher (constante de
//! compilación): nunca se descarga junto con el catálogo ni se acepta como
//! nueva autoridad. La rotación de claves es una actualización firmada del
//! launcher que agrega la clave nueva a `TRUSTED_KEYS` antes de retirar la
//! vieja (ver docs del repo de distribución, "Signature rotation").
//!
//! La firma se comprueba SIEMPRE sobre los bytes crudos recibidos — jamás
//! sobre JSON reserializado.

use base64::Engine;
use ring::signature::{ED25519, UnparsedPublicKey};

pub struct TrustedKey {
    pub id: &'static str,
    /// Clave pública Ed25519 cruda (32 bytes) en Base64 estándar —
    /// idéntica al contenido de `keys/current.pub` del repo de distribución.
    pub public_key_b64: &'static str,
}

/// Claves activas. Verificar contra cualquiera de ellas es válido.
pub const TRUSTED_KEYS: &[TrustedKey] = &[TrustedKey {
    id: "tfl-selection-2026-current",
    public_key_b64: "EfZhA1AVSumBvuI40hQ2o54C5zFGK8jRV/a05PCf/kg=",
}];

/// Tamaño de una firma Ed25519 cruda.
const SIGNATURE_LEN: usize = 64;

#[derive(Debug, PartialEq, Eq)]
pub enum SignatureError {
    /// El archivo `.sig` no es Base64 estándar de 64 bytes.
    Malformed,
    /// Ninguna clave de confianza válida la firmó.
    Invalid,
    /// Hay una clave de confianza mal formada en el binario (bug de build).
    BadTrustedKey,
}

impl std::fmt::Display for SignatureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Malformed => write!(f, "la firma tiene un formato inválido"),
            Self::Invalid => write!(f, "la firma no coincide con ninguna clave de confianza"),
            Self::BadTrustedKey => write!(f, "clave de confianza interna inválida"),
        }
    }
}

fn decode_signature(sig_file: &[u8]) -> Result<Vec<u8>, SignatureError> {
    // El `.sig` publicado termina en salto de línea; se ignora el espacio en
    // los extremos pero NO se acepta basura intermedia.
    let text = std::str::from_utf8(sig_file).map_err(|_| SignatureError::Malformed)?;
    let raw = base64::engine::general_purpose::STANDARD
        .decode(text.trim())
        .map_err(|_| SignatureError::Malformed)?;
    if raw.len() != SIGNATURE_LEN {
        return Err(SignatureError::Malformed);
    }
    Ok(raw)
}

/// Verifica la firma detached de `message` contra un conjunto de claves.
/// Devuelve el id de la clave que firmó.
pub fn verify_with_keys<'k>(
    message: &[u8],
    sig_file: &[u8],
    keys: &'k [TrustedKey],
) -> Result<&'k str, SignatureError> {
    let signature = decode_signature(sig_file)?;
    for key in keys {
        let public = base64::engine::general_purpose::STANDARD
            .decode(key.public_key_b64.trim())
            .map_err(|_| SignatureError::BadTrustedKey)?;
        if public.len() != 32 {
            return Err(SignatureError::BadTrustedKey);
        }
        if UnparsedPublicKey::new(&ED25519, &public)
            .verify(message, &signature)
            .is_ok()
        {
            return Ok(key.id);
        }
    }
    Err(SignatureError::Invalid)
}

/// Verifica contra las claves embebidas en el launcher.
pub fn verify(message: &[u8], sig_file: &[u8]) -> Result<&'static str, SignatureError> {
    verify_with_keys(message, sig_file, TRUSTED_KEYS)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ring::signature::{Ed25519KeyPair, KeyPair};

    const CATALOG: &[u8] = include_bytes!("testdata/catalog.json");
    const CATALOG_SIG: &[u8] = include_bytes!("testdata/catalog.json.sig");
    const MANIFEST: &[u8] = include_bytes!("testdata/manifest-1.8.9.json");
    const MANIFEST_SIG: &[u8] = include_bytes!("testdata/manifest-1.8.9.json.sig");
    const PUBLISHED_PUB: &str = include_str!("testdata/current.pub");

    #[test]
    fn embedded_key_matches_the_published_key_file() {
        assert_eq!(TRUSTED_KEYS[0].public_key_b64, PUBLISHED_PUB.trim());
    }

    #[test]
    fn real_published_catalog_and_manifest_verify() {
        assert_eq!(
            verify(CATALOG, CATALOG_SIG),
            Ok("tfl-selection-2026-current")
        );
        assert_eq!(
            verify(MANIFEST, MANIFEST_SIG),
            Ok("tfl-selection-2026-current")
        );
    }

    #[test]
    fn modified_bytes_fail() {
        let mut tampered = CATALOG.to_vec();
        let pos = tampered.iter().position(|b| *b == b'1').unwrap();
        tampered[pos] = b'2';
        assert_eq!(verify(&tampered, CATALOG_SIG), Err(SignatureError::Invalid));
        // Un espacio extra también cambia los bytes: no se "normaliza".
        let mut spaced = CATALOG.to_vec();
        spaced.push(b' ');
        assert_eq!(verify(&spaced, CATALOG_SIG), Err(SignatureError::Invalid));
    }

    #[test]
    fn signature_of_another_document_fails() {
        assert_eq!(verify(CATALOG, MANIFEST_SIG), Err(SignatureError::Invalid));
    }

    #[test]
    fn malformed_signatures_are_rejected() {
        for bad in [
            &b""[..],
            b"no es base64 !!!",
            b"AAAA",
            // Base64 válido pero de 63 bytes en lugar de 64.
            b"DtLKxSG2Y5oQNFSEfxJBNeCaycFy36ghp//AsH1p+WKlM5fPk1eicO+grWukuaNYPMrjh9oTu4PScjrenYh",
        ] {
            assert_eq!(verify(CATALOG, bad), Err(SignatureError::Malformed), "{bad:?}");
        }
        assert_eq!(verify(CATALOG, &[0xff, 0xfe]), Err(SignatureError::Malformed));
    }

    #[test]
    fn signature_from_an_untrusted_key_fails() {
        let rng = ring::rand::SystemRandom::new();
        let pkcs8 = Ed25519KeyPair::generate_pkcs8(&rng).unwrap();
        let pair = Ed25519KeyPair::from_pkcs8(pkcs8.as_ref()).unwrap();
        let sig = base64::engine::general_purpose::STANDARD.encode(pair.sign(CATALOG).as_ref());
        // Firma criptográficamente correcta, pero de un atacante.
        assert_eq!(
            verify(CATALOG, sig.as_bytes()),
            Err(SignatureError::Invalid)
        );
        // Y verifica solo si esa clave se agrega explícitamente al set.
        let attacker_pub =
            base64::engine::general_purpose::STANDARD.encode(pair.public_key().as_ref());
        let leaked: &'static str = Box::leak(attacker_pub.into_boxed_str());
        let keys = [TrustedKey {
            id: "test",
            public_key_b64: leaked,
        }];
        assert_eq!(verify_with_keys(CATALOG, sig.as_bytes(), &keys), Ok("test"));
    }

    #[test]
    fn broken_trusted_key_is_reported_not_trusted() {
        let keys = [TrustedKey {
            id: "bad",
            public_key_b64: "AAAA",
        }];
        assert_eq!(
            verify_with_keys(CATALOG, CATALOG_SIG, &keys),
            Err(SignatureError::BadTrustedKey)
        );
    }
}
