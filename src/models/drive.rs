use std::path::PathBuf;

/**
 * Representa um dispositivo removível (disco físico inteiro).
 */
#[derive(Clone, Debug)]
pub struct Drive {
    pub path: String,
    pub device_path: PathBuf,
    pub size: u64,
    pub label: Option<String>,
    pub model: Option<String>,
    pub id_type: Option<String>,
    pub is_removable: bool,
    pub mount_points: Vec<PathBuf>,
    pub object_path: String,
    /// Caminho do objeto UDisks2.Drive associado (para localizar partições filhas).
    pub drive_object_path: String,
}

impl Drive {
    /**
     * Retorna o tamanho formatado do dispositivo.
     */
    pub fn formatted_size(&self) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;

        if self.size >= GB {
            format!("{:.1} GB", self.size as f64 / GB as f64)
        } else if self.size >= MB {
            format!("{:.1} MB", self.size as f64 / MB as f64)
        } else if self.size >= KB {
            format!("{:.1} KB", self.size as f64 / KB as f64)
        } else {
            format!("{} B", self.size)
        }
    }

    /**
     * Nome para exibição: modelo do fabricante, label ou nome do dispositivo.
     */
    pub fn display_name(&self) -> String {
        if let Some(model) = &self.model {
            let trimmed = model.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
        self.label
            .clone()
            .unwrap_or_else(|| self.path.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::Drive;
    use std::path::PathBuf;

    fn sample_drive() -> Drive {
        Drive {
            path: "sdd".to_string(),
            device_path: PathBuf::from("/dev/sdd"),
            size: 57_700_000_000,
            label: Some("OLDLABEL".to_string()),
            model: None,
            id_type: None,
            is_removable: true,
            mount_points: Vec::new(),
            object_path: "/org/freedesktop/UDisks2/block_devices/sdd".to_string(),
            drive_object_path: "/org/freedesktop/UDisks2/drives/Test".to_string(),
        }
    }

    #[test]
    fn tamanho_formatado_em_gb() {
        let drive = sample_drive();
        assert_eq!(drive.formatted_size(), "53.7 GB");
    }

    #[test]
    fn tamanho_formatado_bytes_kb_mb() {
        let mut drive = sample_drive();
        drive.size = 512;
        assert_eq!(drive.formatted_size(), "512 B");
        drive.size = 2048;
        assert_eq!(drive.formatted_size(), "2.0 KB");
        drive.size = 5 * 1024 * 1024;
        assert_eq!(drive.formatted_size(), "5.0 MB");
    }

    #[test]
    fn nome_exibicao_prioriza_modelo() {
        let mut drive = sample_drive();
        drive.model = Some("Kingston DataTraveler 3.0".to_string());
        assert_eq!(drive.display_name(), "Kingston DataTraveler 3.0");
    }

    #[test]
    fn nome_exibicao_usa_label_ou_dispositivo() {
        let mut drive = sample_drive();
        assert_eq!(drive.display_name(), "OLDLABEL");
        drive.label = None;
        assert_eq!(drive.display_name(), "sdd");
    }

    #[test]
    fn nome_exibicao_ignora_modelo_vazio() {
        let mut drive = sample_drive();
        drive.model = Some("   ".to_string());
        assert_eq!(drive.display_name(), "OLDLABEL");
    }
}
