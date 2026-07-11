use crate::services::FilesystemType;

const FORBIDDEN_LABEL_CHARS: &[char] = &['\\', '/', ':', '*', '?', '"', '<', '>', '|'];

fn find_forbidden_chars(text: &str) -> Vec<char> {
    let mut found: Vec<char> = text
        .chars()
        .filter(|c| FORBIDDEN_LABEL_CHARS.contains(c))
        .collect();
    found.sort();
    found.dedup();
    found
}

fn format_forbidden_chars_message(forbidden: &[char]) -> String {
    let chars_list: String = forbidden
        .iter()
        .map(|c| {
            if *c == '\\' {
                "\\ (barra invertida)".to_string()
            } else if *c == '"' {
                "\" (aspas)".to_string()
            } else {
                format!("'{c}'")
            }
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "O nome do volume não pode conter o(s) caractere(s): {chars_list}"
    )
}

/// Valida o nome do volume para o sistema de arquivos escolhido.
pub fn validate_volume_label(
    text: &str,
    fs_type: FilesystemType,
) -> Result<Option<String>, String> {
    let text = text.trim();
    if text.is_empty() {
        return Ok(None);
    }

    let forbidden = find_forbidden_chars(text);
    if !forbidden.is_empty() {
        return Err(format_forbidden_chars_message(&forbidden));
    }

    let max_len = fs_type.max_label_length() as usize;
    if text.chars().count() > max_len {
        return Err(format!(
            "O nome do volume não pode ter mais de {max_len} caracteres para {}.",
            fs_type.display_name()
        ));
    }

    Ok(Some(text.to_string()))
}

/// Trunca o texto ao limite do sistema de arquivos (por contagem de caracteres Unicode).
pub fn truncate_to_max(text: &str, fs_type: FilesystemType) -> String {
    let max_len = fs_type.max_label_length() as usize;
    text.chars().take(max_len).collect()
}

/// Texto de ajuda exibido abaixo do campo de nome do volume.
pub fn label_hint_for(fs_type: FilesystemType) -> String {
    format!(
        "Máximo {} caracteres ({})",
        fs_type.max_label_length(),
        fs_type.display_name()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::FilesystemType;

    #[test]
    fn permite_label_vazio() {
        assert_eq!(validate_volume_label("", FilesystemType::Fat32).unwrap(), None);
        assert_eq!(
            validate_volume_label("   ", FilesystemType::ExFat).unwrap(),
            None
        );
    }

    #[test]
    fn label_valido_remove_espacos() {
        assert_eq!(
            validate_volume_label("  MeuPen  ", FilesystemType::Fat32).unwrap(),
            Some("MeuPen".to_string())
        );
    }

    #[test]
    fn rejeita_caracteres_proibidos() {
        let err = validate_volume_label("a/b", FilesystemType::Fat32).unwrap_err();
        assert!(err.contains("caractere(s)"));
        assert!(err.contains("/"));
    }

    #[test]
    fn fat32_limite_11_caracteres() {
        assert!(validate_volume_label("12345678901", FilesystemType::Fat32).is_ok());
        let err = validate_volume_label("123456789012", FilesystemType::Fat32).unwrap_err();
        assert!(err.contains("11"));
        assert!(err.contains("FAT32"));
    }

    #[test]
    fn exfat_limite_11_caracteres() {
        let ok = "a".repeat(11);
        let long = "a".repeat(12);
        assert!(validate_volume_label(&ok, FilesystemType::ExFat).is_ok());
        assert!(validate_volume_label(&long, FilesystemType::ExFat).is_err());
    }

    #[test]
    fn fat32_e_exfat_compartilham_limite_11() {
        assert_eq!(
            FilesystemType::Fat32.max_label_length(),
            FilesystemType::ExFat.max_label_length()
        );
    }

    #[test]
    fn truncate_to_max_respeita_limite() {
        assert_eq!(
            truncate_to_max("123456789012", FilesystemType::ExFat),
            "12345678901"
        );
    }

    #[test]
    fn ext4_limite_16_caracteres() {
        let ok = "a".repeat(16);
        let long = "a".repeat(17);
        assert!(validate_volume_label(&ok, FilesystemType::Ext4).is_ok());
        assert!(validate_volume_label(&long, FilesystemType::Ext4).is_err());
    }

    #[test]
    fn truncate_unicode_por_caracteres_nao_bytes() {
        let text = "ação1234567"; // 11 chars with ç and ã
        assert_eq!(truncate_to_max(text, FilesystemType::Fat32), text);
        let long = format!("{text}x");
        assert_eq!(
            truncate_to_max(&long, FilesystemType::Fat32).chars().count(),
            11
        );
    }

    #[test]
    fn label_hint_para_cada_fs() {
        assert!(label_hint_for(FilesystemType::Fat32).contains("FAT32"));
        assert!(label_hint_for(FilesystemType::ExFat).contains("exFAT"));
        assert!(label_hint_for(FilesystemType::Ntfs).contains("NTFS"));
        assert!(label_hint_for(FilesystemType::Ext4).contains("ext4"));
    }

    #[test]
    fn ntfs_limite_32_caracteres() {
        let ok = "a".repeat(32);
        let long = "a".repeat(33);
        assert!(validate_volume_label(&ok, FilesystemType::Ntfs).is_ok());
        let err = validate_volume_label(&long, FilesystemType::Ntfs).unwrap_err();
        assert!(err.contains("32"));
        assert!(err.contains("NTFS"));
    }
}
