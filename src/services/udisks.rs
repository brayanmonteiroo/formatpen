use crate::models::Drive;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::PathBuf;
use zbus::blocking::Connection;
use zbus::zvariant::{OwnedObjectPath, OwnedValue};

const UDISKS2_PATH: &str = "/org/freedesktop/UDisks2";
const UDISKS2_SERVICE: &str = "org.freedesktop.UDisks2";
const BLOCK_INTERFACE: &str = "org.freedesktop.UDisks2.Block";
const DRIVE_INTERFACE: &str = "org.freedesktop.UDisks2.Drive";
const FILESYSTEM_INTERFACE: &str = "org.freedesktop.UDisks2.Filesystem";
const OBJECT_MANAGER_INTERFACE: &str = "org.freedesktop.DBus.ObjectManager";
const PARTITION_INTERFACE: &str = "org.freedesktop.UDisks2.Partition";

type ManagedObjects = HashMap<
    OwnedObjectPath,
    HashMap<String, HashMap<String, OwnedValue>>,
>;

pub struct UDisksService;

/**
 * Implementação do serviço de UDisks.
 */
impl UDisksService {
    /**
     * Lista os dispositivos removíveis.
     */
    pub fn list_removable_drives() -> Result<Vec<Drive>> {
        let connection = Connection::system().context("Falha ao conectar ao D-Bus do sistema")?;

        let reply = connection
            .call_method(
                Some(UDISKS2_SERVICE),
                UDISKS2_PATH,
                Some(OBJECT_MANAGER_INTERFACE),
                "GetManagedObjects",
                &(),
            )
            .context("Falha ao obter objetos gerenciados do UDisks2")?;

        let managed: ManagedObjects = reply
            .body()
            .deserialize()
            .context("Falha ao ler resposta do UDisks2")?;

        let drive_cache = Self::extract_drives(&managed);
        let mut partitions_by_drive: HashMap<String, Vec<Drive>> = HashMap::new();
        let mut whole_disks_by_drive: HashMap<String, Vec<Drive>> = HashMap::new();

        for (path, interfaces) in &managed {
            let path_str = path.as_str();
            if !path_str.contains("block_devices/") {
                continue;
            }

            let Some(block_props) = interfaces.get(BLOCK_INTERFACE) else {
                continue;
            };

            let hint_system = Self::get_bool(block_props, "HintSystem").unwrap_or(true);
            let hint_ignore = Self::get_bool(block_props, "HintIgnore").unwrap_or(false);
            if hint_system || hint_ignore {
                continue;
            }

            let drive_path = Self::get_object_path(block_props, "Drive").unwrap_or_default();
            let is_removable = drive_cache
                .get(&drive_path)
                .copied()
                .unwrap_or(false);

            if !is_removable {
                continue;
            }

            let size = Self::get_u64(block_props, "Size").unwrap_or(0);
            let id_label = Self::get_str(block_props, "IdLabel");
            let id_type = Self::get_str(block_props, "IdType");
            let device = Self::get_byte_array_as_path(block_props, "PreferredDevice")
                .or_else(|| Self::get_byte_array_as_path(block_props, "Device"))
                .unwrap_or_else(|| PathBuf::from("/dev/unknown"));
            let mount_points = interfaces
                .get(FILESYSTEM_INTERFACE)
                .map_or(Vec::new(), |_| Vec::new());

            let path_name = device
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();

            let drive = Drive {
                path: path_name,
                device_path: device,
                size,
                label: id_label,
                id_type,
                is_removable,
                mount_points,
                object_path: path_str.to_string(),
            };

            let is_partition = interfaces.contains_key(PARTITION_INTERFACE);
            if is_partition {
                partitions_by_drive
                    .entry(drive_path.clone())
                    .or_default()
                    .push(drive);
            } else {
                whole_disks_by_drive
                    .entry(drive_path)
                    .or_default()
                    .push(drive);
            }
        }

        let mut drives = Vec::new();
        for partitions in partitions_by_drive.values() {
            drives.extend(partitions.clone());
        }
        for (drive_path, whole_disks) in &whole_disks_by_drive {
            if !partitions_by_drive.contains_key(drive_path) {
                drives.extend(whole_disks.clone());
            }
        }

        drives.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(drives)
    }

    /**
     * Extrai os dispositivos removíveis.
     */
    fn extract_drives(managed: &ManagedObjects) -> HashMap<String, bool> {
        let mut cache = HashMap::new();
        for (path, interfaces) in managed {
            if path.as_str().contains("drives/") {
                if let Some(props) = interfaces.get(DRIVE_INTERFACE) {
                    let removable = Self::get_bool(props, "Removable").unwrap_or(false);
                    cache.insert(path.as_str().to_string(), removable);
                }
            }
        }
        cache
    }

    /**
     * Retorna o valor booleano.
     */
    fn get_bool(props: &HashMap<String, OwnedValue>, key: &str) -> Option<bool> {
        props.get(key).and_then(|v| bool::try_from(v).ok())
    }

    /**
     * Retorna o valor inteiro.
     */
    fn get_u64(props: &HashMap<String, OwnedValue>, key: &str) -> Option<u64> {
        props.get(key).and_then(|v| {
            u64::try_from(v).ok()
        })
    }

    /**
     * Retorna o valor string.
     */
    fn get_str(props: &HashMap<String, OwnedValue>, key: &str) -> Option<String> {
        props.get(key).and_then(|v| {
            <&str>::try_from(v).ok().map(|s| s.to_string())
        })
    }

    /**
     * Retorna o valor objeto path.
     */
    fn get_object_path(props: &HashMap<String, OwnedValue>, key: &str) -> Option<String> {
        props.get(key).and_then(|v| {
            <&zbus::zvariant::ObjectPath<'_>>::try_from(v)
                .ok()
                .map(|p| p.as_str().to_string())
        })
    }

    /**
     * Retorna o valor byte array como path.
     */
    fn get_byte_array_as_path(props: &HashMap<String, OwnedValue>, key: &str) -> Option<PathBuf> {
        props.get(key).and_then(|v| {
            let value = v.downcast_ref::<zbus::zvariant::Value<'_>>();
            if let Ok(zbus::zvariant::Value::Array(array)) = value {
                let bytes: std::result::Result<Vec<u8>, _> = array.try_clone()
                    .ok()
                    .and_then(|a| a.try_into().ok())
                    .ok_or(());
                if let Ok(bytes) = bytes {
                    let trimmed = if bytes.last() == Some(&0) {
                        &bytes[..bytes.len() - 1]
                    } else {
                        &bytes
                    };
                    return std::str::from_utf8(trimmed).ok().map(PathBuf::from);
                }
            }
            None
        })
    }
}
