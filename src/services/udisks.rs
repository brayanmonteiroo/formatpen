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
const OBJECT_MANAGER_INTERFACE: &str = "org.freedesktop.DBus.ObjectManager";
const PARTITION_INTERFACE: &str = "org.freedesktop.UDisks2.Partition";

type ManagedObjects = HashMap<
    OwnedObjectPath,
    HashMap<String, HashMap<String, OwnedValue>>,
>;

struct DriveMeta {
    removable: bool,
    model: Option<String>,
}

pub struct UDisksService;

impl UDisksService {
    /**
     * Lista discos removíveis inteiros (não partições individuais).
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

        let drive_cache = Self::extract_drive_meta(&managed);
        let mut drives = Vec::new();

        for (path, interfaces) in &managed {
            let path_str = path.as_str();
            if !path_str.contains("block_devices/") {
                continue;
            }

            if interfaces.contains_key(PARTITION_INTERFACE) {
                continue;
            }

            let Some(block_props) = interfaces.get(BLOCK_INTERFACE) else {
                continue;
            };

            let hint_system = Self::get_bool(block_props, "HintSystem").unwrap_or(true);
            let hint_ignore = Self::get_bool(block_props, "HintIgnore").unwrap_or(false);
            let hint_partitionable =
                Self::get_bool(block_props, "HintPartitionable").unwrap_or(false);
            let read_only = Self::get_bool(block_props, "ReadOnly").unwrap_or(false);

            if hint_system || hint_ignore || !hint_partitionable || read_only {
                continue;
            }

            let drive_object_path =
                Self::get_object_path(block_props, "Drive").unwrap_or_default();
            let drive_meta = drive_cache.get(&drive_object_path);

            let is_removable = drive_meta.map(|m| m.removable).unwrap_or(false);
            if !is_removable {
                continue;
            }

            let model = drive_meta.and_then(|m| m.model.clone());
            let size = Self::get_u64(block_props, "Size").unwrap_or(0);
            let id_label = Self::get_str(block_props, "IdLabel");
            let id_type = Self::get_str(block_props, "IdType");
            let device = Self::get_byte_array_as_path(block_props, "PreferredDevice")
                .or_else(|| Self::get_byte_array_as_path(block_props, "Device"))
                .unwrap_or_else(|| PathBuf::from("/dev/unknown"));

            let path_name = device
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();

            drives.push(Drive {
                path: path_name,
                device_path: device,
                size,
                label: id_label,
                model,
                id_type,
                is_removable,
                mount_points: Vec::new(),
                object_path: path_str.to_string(),
                drive_object_path: drive_object_path.clone(),
            });
        }

        drives.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(drives)
    }

    fn extract_drive_meta(managed: &ManagedObjects) -> HashMap<String, DriveMeta> {
        let mut cache = HashMap::new();
        for (path, interfaces) in managed {
            if !path.as_str().contains("drives/") {
                continue;
            }
            let Some(props) = interfaces.get(DRIVE_INTERFACE) else {
                continue;
            };
            let removable = Self::get_bool(props, "Removable").unwrap_or(false);
            let vendor = Self::get_str(props, "Vendor").unwrap_or_default();
            let model_name = Self::get_str(props, "Model").unwrap_or_default();
            let model = combine_vendor_model(&vendor, &model_name);
            cache.insert(
                path.as_str().to_string(),
                DriveMeta { removable, model },
            );
        }
        cache
    }

    fn get_bool(props: &HashMap<String, OwnedValue>, key: &str) -> Option<bool> {
        props.get(key).and_then(|v| bool::try_from(v).ok())
    }

    fn get_u64(props: &HashMap<String, OwnedValue>, key: &str) -> Option<u64> {
        props.get(key).and_then(|v| u64::try_from(v).ok())
    }

    fn get_str(props: &HashMap<String, OwnedValue>, key: &str) -> Option<String> {
        props
            .get(key)
            .and_then(|v| <&str>::try_from(v).ok().map(|s| s.to_string()))
    }

    fn get_object_path(props: &HashMap<String, OwnedValue>, key: &str) -> Option<String> {
        props.get(key).and_then(|v| {
            <&zbus::zvariant::ObjectPath<'_>>::try_from(v)
                .ok()
                .map(|p| p.as_str().to_string())
        })
    }

    fn get_byte_array_as_path(props: &HashMap<String, OwnedValue>, key: &str) -> Option<PathBuf> {
        props.get(key).and_then(|v| {
            let value = v.downcast_ref::<zbus::zvariant::Value<'_>>();
            if let Ok(zbus::zvariant::Value::Array(array)) = value {
                let bytes: std::result::Result<Vec<u8>, _> = array
                    .try_clone()
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

/// Combina fabricante e modelo para exibição na lista de dispositivos.
pub fn combine_vendor_model(vendor: &str, model: &str) -> Option<String> {
    let vendor = vendor.trim();
    let model = model.trim();
    if vendor.is_empty() && model.is_empty() {
        return None;
    }
    if vendor.is_empty() {
        return Some(model.to_string());
    }
    if model.is_empty() {
        return Some(vendor.to_string());
    }
    if model.starts_with(vendor) {
        return Some(model.to_string());
    }
    Some(format!("{vendor} {model}"))
}

#[cfg(test)]
mod tests {
    use super::combine_vendor_model;

    #[test]
    fn fabricante_e_modelo_vazios_retorna_none() {
        assert_eq!(combine_vendor_model("", ""), None);
        assert_eq!(combine_vendor_model("  ", "  "), None);
    }

    #[test]
    fn so_fabricante_ou_so_modelo() {
        assert_eq!(
            combine_vendor_model("Kingston", ""),
            Some("Kingston".to_string())
        );
        assert_eq!(
            combine_vendor_model("", "DataTraveler 3.0"),
            Some("DataTraveler 3.0".to_string())
        );
    }

    #[test]
    fn nao_duplica_fabricante_no_modelo() {
        assert_eq!(
            combine_vendor_model("SanDisk", "SanDisk Ultra"),
            Some("SanDisk Ultra".to_string())
        );
    }

    #[test]
    fn junta_fabricante_e_modelo() {
        assert_eq!(
            combine_vendor_model("Kingston", "DataTraveler 3.0"),
            Some("Kingston DataTraveler 3.0".to_string())
        );
    }
}
