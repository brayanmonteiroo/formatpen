use std::path::PathBuf;

/**
 * Representa um dispositivo removível. Ele possui o caminho do dispositivo, o caminho do dispositivo físico, o tamanho, o label, o tipo de ID, se é removível, os pontos de montagem e o caminho do objeto.
 */
#[derive(Clone, Debug)]
pub struct Drive {
    pub path: String,
    pub device_path: PathBuf,
    pub size: u64,
    pub label: Option<String>,
    pub id_type: Option<String>,
    pub is_removable: bool,
    pub mount_points: Vec<PathBuf>,
    pub object_path: String,
}

/**
 * Implementação do dispositivo removível.
 */
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
    * Retorna o nome do dispositivo.
    * Se o label do dispositivo não estiver vazio, retorna o label, caso contrário, retorna o nome do dispositivo.
    */
    pub fn display_name(&self) -> String {
        self.label
            .clone()
            .unwrap_or_else(|| self.path.clone())
    }
}
