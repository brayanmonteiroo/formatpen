use anyhow::{Context, Result};
use std::collections::HashMap;
use zbus::blocking::Connection;
use zbus::zvariant::Value;

const UDISKS2_SERVICE: &str = "org.freedesktop.UDisks2";
const BLOCK_INTERFACE: &str = "org.freedesktop.UDisks2.Block";
const FILESYSTEM_INTERFACE: &str = "org.freedesktop.UDisks2.Filesystem";

/**
 * Tipos de sistema de arquivos suportados.
 */
#[derive(Clone, Copy, Debug)]
pub enum FilesystemType {
    Fat32,
    ExFat,
    Ntfs,
    Ext4,
}

/**
 * Implementação dos tipos de sistema de arquivos.
 */
impl FilesystemType {
    /**
     * Retorna o nome do sistema de arquivos para o UDisks2.
     */
    pub fn udisks2_name(&self) -> &'static str {
        match self {
            Self::Fat32 => "vfat",
            Self::ExFat => "exfat",
            Self::Ntfs => "ntfs",
            Self::Ext4 => "ext4",
        }
    }

    /**
     * Retorna o nome do sistema de arquivos para exibição.
     */
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Fat32 => "FAT32",
            Self::ExFat => "exFAT",
            Self::Ntfs => "NTFS",
            Self::Ext4 => "ext4",
        }
    }

    /**
     * Retorna o número máximo de caracteres permitidos no nome do volume.
     * FAT32: 11, exFAT: 15, NTFS: 32, ext4: 16.
     */
    pub fn max_label_length(&self) -> u32 {
        match self {
            Self::Fat32 => 11,
            Self::ExFat => 15,
            Self::Ntfs => 32,
            Self::Ext4 => 16,
        }
    }

    /**
     * Retorna todos os tipos de sistema de arquivos.
     */
    pub fn all() -> &'static [Self] {
        &[Self::Fat32, Self::ExFat, Self::Ntfs, Self::Ext4]
    }
}

/**
 * Serviço para formatação de dispositivos.
 */
pub struct FormatService;

/**
 * Implementação do serviço para formatação de dispositivos.
 */
impl FormatService {
    /**
     * Desmonta o dispositivo se necessário.
     */
    fn unmount_if_needed(connection: &Connection, object_path: &str) -> Result<()> {
        let options: HashMap<&str, Value> = HashMap::new();
        let _ = connection.call_method(
            Some(UDISKS2_SERVICE),
            object_path,
            Some(FILESYSTEM_INTERFACE),
            "Unmount",
            &options,
        );
        Ok(())
    }

    /**
     * Formata o dispositivo.
     */
    pub fn format(
        object_path: &str,
        fs_type: FilesystemType,
        label: Option<&str>,
    ) -> Result<()> {
        let connection =
            Connection::system().context("Falha ao conectar ao D-Bus do sistema")?;

        Self::unmount_if_needed(&connection, object_path)
            .context("Falha ao desmontar o dispositivo")?;

        let mut options: HashMap<&str, Value> = HashMap::new();
        if let Some(l) = label {
            if !l.is_empty() {
                options.insert("label", Value::from(l));
            }
        }

        connection
            .call_method(
                Some(UDISKS2_SERVICE),
                object_path,
                Some(BLOCK_INTERFACE),
                "Format",
                &(fs_type.udisks2_name(), options),
            )
            .context("Falha ao chamar Format no UDisks2")?;

        Ok(())
    }
}
