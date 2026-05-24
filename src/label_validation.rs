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
    fn exfat_limite_15_caracteres() {
        let ok = "a".repeat(15);
        let long = "a".repeat(16);
        assert!(validate_volume_label(&ok, FilesystemType::ExFat).is_ok());
        assert!(validate_volume_label(&long, FilesystemType::ExFat).is_err());
    }

    #[test]
    fn ext4_limite_16_caracteres() {
        let ok = "a".repeat(16);
        let long = "a".repeat(17);
        assert!(validate_volume_label(&ok, FilesystemType::Ext4).is_ok());
        assert!(validate_volume_label(&long, FilesystemType::Ext4).is_err());
    }
}
