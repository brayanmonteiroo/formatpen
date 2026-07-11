use anyhow::{Context, Result};
use std::collections::HashMap;
use zbus::blocking::Connection;
use zbus::zvariant::{OwnedObjectPath, OwnedValue, Value};

const UDISKS2_PATH: &str = "/org/freedesktop/UDisks2";
const UDISKS2_SERVICE: &str = "org.freedesktop.UDisks2";
const BLOCK_INTERFACE: &str = "org.freedesktop.UDisks2.Block";
const FILESYSTEM_INTERFACE: &str = "org.freedesktop.UDisks2.Filesystem";
const OBJECT_MANAGER_INTERFACE: &str = "org.freedesktop.DBus.ObjectManager";
const PARTITION_INTERFACE: &str = "org.freedesktop.UDisks2.Partition";
const PARTITION_TABLE_INTERFACE: &str = "org.freedesktop.UDisks2.PartitionTable";

const PARTITION_OFFSET: u64 = 1_048_576;
const PARTITION_SIZE_MAX: u64 = 0;

type ManagedObjects = HashMap<
    OwnedObjectPath,
    HashMap<String, HashMap<String, OwnedValue>>,
>;

#[derive(Clone, Copy, Debug)]
pub enum FilesystemType {
    Fat32,
    ExFat,
    Ntfs,
    Ext4,
}

impl FilesystemType {
    pub fn udisks2_name(&self) -> &'static str {
        match self {
            Self::Fat32 => "vfat",
            Self::ExFat => "exfat",
            Self::Ntfs => "ntfs",
            Self::Ext4 => "ext4",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Fat32 => "FAT32",
            Self::ExFat => "exFAT",
            Self::Ntfs => "NTFS",
            Self::Ext4 => "ext4",
        }
    }

    pub fn partition_table_type(&self) -> &'static str {
        match self {
            Self::Ext4 => "gpt",
            _ => "dos",
        }
    }

    pub fn max_label_length(&self) -> u32 {
        match self {
            Self::Fat32 => 11,
            Self::ExFat => 11,
            Self::Ntfs => 32,
            Self::Ext4 => 16,
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::Fat32, Self::ExFat, Self::Ntfs, Self::Ext4]
    }
}

/// Nome da partição para `CreatePartitionAndFormat` (MBR não aceita nome).
pub fn partition_name_for_create(fs_type: FilesystemType, label: Option<&str>) -> String {
    match fs_type.partition_table_type() {
        "gpt" => label.unwrap_or("").to_string(),
        _ => String::new(),
    }
}

pub struct FormatService;

impl FormatService {
    fn connect() -> Result<Connection> {
        Connection::system().context("Falha ao conectar ao D-Bus do sistema")
    }

    fn get_managed_objects(connection: &Connection) -> Result<ManagedObjects> {
        let reply = connection
            .call_method(
                Some(UDISKS2_SERVICE),
                UDISKS2_PATH,
                Some(OBJECT_MANAGER_INTERFACE),
                "GetManagedObjects",
                &(),
            )
            .context("Falha ao obter objetos gerenciados do UDisks2")?;

        reply
            .body()
            .deserialize()
            .context("Falha ao ler resposta do UDisks2")
    }

    fn unmount_if_needed(connection: &Connection, object_path: &str) {
        let mut options: HashMap<String, Value> = HashMap::new();
        options.insert("force".to_string(), Value::from(true));
        let _ = connection.call_method(
            Some(UDISKS2_SERVICE),
            object_path,
            Some(FILESYSTEM_INTERFACE),
            "Unmount",
            &options,
        );
    }

    fn partition_paths_for_drive(
        managed: &ManagedObjects,
        drive_object_path: &str,
    ) -> Vec<String> {
        partition_paths_for_drive(managed, drive_object_path)
    }

    fn unmount_all_partitions(
        connection: &Connection,
        managed: &ManagedObjects,
        drive_object_path: &str,
    ) {
        for path in Self::partition_paths_for_drive(managed, drive_object_path) {
            Self::unmount_if_needed(connection, &path);
        }
    }

    fn delete_all_partitions(
        connection: &Connection,
        managed: &ManagedObjects,
        drive_object_path: &str,
    ) {
        let mut options: HashMap<String, Value> = HashMap::new();
        options.insert("tear-down".to_string(), Value::from(true));

        let mut paths = Self::partition_paths_for_drive(managed, drive_object_path);
        paths = partition_delete_order(&paths);

        for path in paths {
            let _ = connection.call_method(
                Some(UDISKS2_SERVICE),
                path.as_str(),
                Some(PARTITION_INTERFACE),
                "Delete",
                &options,
            );
        }
    }

    fn format_partition_table(
        connection: &Connection,
        disk_object_path: &str,
        partition_table_type: &str,
    ) -> Result<()> {
        let mut options: HashMap<String, Value> = HashMap::new();
        options.insert("tear-down".to_string(), Value::from(true));

        connection
            .call_method(
                Some(UDISKS2_SERVICE),
                disk_object_path,
                Some(BLOCK_INTERFACE),
                "Format",
                &(partition_table_type, options),
            )
            .context("Falha ao recriar tabela de partições")?;

        Ok(())
    }

    fn create_partition_and_format(
        connection: &Connection,
        disk_object_path: &str,
        fs_type: FilesystemType,
        label: Option<&str>,
    ) -> Result<()> {
        let mut format_options: HashMap<String, Value> = HashMap::new();
        if let Some(l) = label {
            if !l.is_empty() {
                format_options.insert("label".to_string(), Value::from(l));
            }
        }

        let partition_name = partition_name_for_create(fs_type, label);
        let options: HashMap<String, Value> = HashMap::new();

        connection
            .call_method(
                Some(UDISKS2_SERVICE),
                disk_object_path,
                Some(PARTITION_TABLE_INTERFACE),
                "CreatePartitionAndFormat",
                &(
                    PARTITION_OFFSET,
                    PARTITION_SIZE_MAX,
                    "",
                    partition_name.as_str(),
                    options,
                    fs_type.udisks2_name(),
                    format_options,
                ),
            )
            .context("Falha ao criar partição e formatar")?;

        Ok(())
    }

    pub fn format(
        disk_object_path: &str,
        drive_object_path: &str,
        fs_type: FilesystemType,
        label: Option<&str>,
    ) -> Result<()> {
        let connection = Self::connect()?;

        let managed = Self::get_managed_objects(&connection)?;
        Self::unmount_all_partitions(&connection, &managed, drive_object_path);
        Self::delete_all_partitions(&connection, &managed, drive_object_path);

        Self::format_partition_table(
            &connection,
            disk_object_path,
            fs_type.partition_table_type(),
        )?;

        Self::create_partition_and_format(&connection, disk_object_path, fs_type, label)?;

        Ok(())
    }
}

/// Partições UDisks2 associadas a um drive (testável com fixtures).
pub(crate) fn partition_paths_for_drive(
    managed: &ManagedObjects,
    drive_object_path: &str,
) -> Vec<String> {
    let mut paths = Vec::new();
    for (path, interfaces) in managed {
        if !interfaces.contains_key(PARTITION_INTERFACE) {
            continue;
        }
        let Some(block_props) = interfaces.get(BLOCK_INTERFACE) else {
            continue;
        };
        let Some(block_drive) = get_object_path(block_props, "Drive") else {
            continue;
        };
        if block_drive == drive_object_path {
            paths.push(path.as_str().to_string());
        }
    }
    paths.sort();
    paths
}

/// Ordem de exclusão de partições (maior path primeiro).
pub(crate) fn partition_delete_order(paths: &[String]) -> Vec<String> {
    let mut ordered = paths.to_vec();
    ordered.sort_by(|a, b| b.cmp(a));
    ordered
}

fn get_object_path(props: &HashMap<String, OwnedValue>, key: &str) -> Option<String> {
    props.get(key).and_then(|v| {
        <&zbus::zvariant::ObjectPath<'_>>::try_from(v)
            .ok()
            .map(|p| p.as_str().to_string())
    })
}

#[cfg(test)]
mod tests {
    use super::{partition_delete_order, partition_name_for_create, partition_paths_for_drive, FilesystemType};
    use crate::test_support::removable_disk_with_partition;

    #[test]
    fn tipo_fs_mapeia_nome_udisks() {
        assert_eq!(FilesystemType::Fat32.udisks2_name(), "vfat");
        assert_eq!(FilesystemType::ExFat.udisks2_name(), "exfat");
        assert_eq!(FilesystemType::Ntfs.udisks2_name(), "ntfs");
        assert_eq!(FilesystemType::Ext4.udisks2_name(), "ext4");
    }

    #[test]
    fn fat_exfat_ntfs_usam_mbr() {
        assert_eq!(FilesystemType::Fat32.partition_table_type(), "dos");
        assert_eq!(FilesystemType::ExFat.partition_table_type(), "dos");
        assert_eq!(FilesystemType::Ntfs.partition_table_type(), "dos");
        assert_eq!(FilesystemType::Ext4.partition_table_type(), "gpt");
    }

    #[test]
    fn mbr_nao_usa_label_como_nome_particao() {
        assert_eq!(
            partition_name_for_create(FilesystemType::ExFat, Some("Teste2")),
            ""
        );
        assert_eq!(
            partition_name_for_create(FilesystemType::Fat32, Some("MEUPENDRIVE")),
            ""
        );
    }

    #[test]
    fn gpt_permite_nome_particao_do_label() {
        assert_eq!(
            partition_name_for_create(FilesystemType::Ext4, Some("dados")),
            "dados"
        );
        assert_eq!(partition_name_for_create(FilesystemType::Ext4, None), "");
    }

    #[test]
    fn partition_paths_for_drive_lista_particoes() {
        let managed = removable_disk_with_partition();
        let drive_path = "/org/freedesktop/UDisks2/drives/Kingston_xxx";
        let paths = partition_paths_for_drive(&managed, drive_path);
        assert_eq!(paths.len(), 1);
        assert!(paths[0].contains("sde1"));
    }

    #[test]
    fn partition_delete_order_reversa() {
        let paths = vec![
            "/org/freedesktop/UDisks2/block_devices/sde1".to_string(),
            "/org/freedesktop/UDisks2/block_devices/sde2".to_string(),
            "/org/freedesktop/UDisks2/block_devices/sde3".to_string(),
        ];
        let ordered = partition_delete_order(&paths);
        assert_eq!(ordered[0], paths[2]);
        assert_eq!(ordered[1], paths[1]);
        assert_eq!(ordered[2], paths[0]);
    }

    #[test]
    fn max_label_length_por_fs() {
        assert_eq!(FilesystemType::Fat32.max_label_length(), 11);
        assert_eq!(FilesystemType::ExFat.max_label_length(), 11);
        assert_eq!(FilesystemType::Ntfs.max_label_length(), 32);
        assert_eq!(FilesystemType::Ext4.max_label_length(), 16);
    }
}
