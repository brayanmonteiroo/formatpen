//! Fixtures UDisks2 para testes unitários (`ManagedObjects` simulados).

use std::collections::HashMap;
use zbus::zvariant::{ObjectPath, OwnedObjectPath, OwnedValue, Value};

pub type ManagedObjects = HashMap<
    OwnedObjectPath,
    HashMap<String, HashMap<String, OwnedValue>>,
>;

const BLOCK: &str = "org.freedesktop.UDisks2.Block";
const DRIVE_IF: &str = "org.freedesktop.UDisks2.Drive";
const PARTITION: &str = "org.freedesktop.UDisks2.Partition";

fn owned(value: Value<'_>) -> OwnedValue {
    value.try_into().expect("valor OwnedValue válido")
}

fn block_path(name: &str) -> OwnedObjectPath {
    ObjectPath::from_str_unchecked(&format!(
        "/org/freedesktop/UDisks2/block_devices/{name}"
    ))
    .into()
}

fn drive_path(name: &str) -> OwnedObjectPath {
    ObjectPath::from_str_unchecked(&format!("/org/freedesktop/UDisks2/drives/{name}"))
        .into()
}

fn device_bytes(dev: &str) -> OwnedValue {
    let mut bytes: Vec<u8> = dev.bytes().collect();
    bytes.push(0);
    owned(Value::from(bytes))
}

pub struct BlockBuilder {
    props: HashMap<String, OwnedValue>,
    is_partition: bool,
}

impl BlockBuilder {
    pub fn whole_disk(name: &str, drive_name: &str) -> Self {
        let mut props = HashMap::new();
        props.insert("HintSystem".into(), owned(Value::from(false)));
        props.insert("HintIgnore".into(), owned(Value::from(false)));
        props.insert("HintPartitionable".into(), owned(Value::from(true)));
        props.insert("ReadOnly".into(), owned(Value::from(false)));
        props.insert(
            "Drive".into(),
            owned(Value::from(ObjectPath::from_str_unchecked(&format!(
                "/org/freedesktop/UDisks2/drives/{drive_name}"
            )))),
        );
        props.insert("Size".into(), owned(Value::from(60_000_000_000_u64)));
        props.insert("PreferredDevice".into(), device_bytes(&format!("/dev/{name}")));
        Self {
            props,
            is_partition: false,
        }
    }

    pub fn partition(name: &str, drive_name: &str) -> Self {
        let mut b = Self::whole_disk(name, drive_name);
        b.is_partition = true;
        b
    }

    pub fn hint_system(mut self, value: bool) -> Self {
        self.props.insert("HintSystem".into(), owned(Value::from(value)));
        self
    }

    pub fn hint_ignore(mut self, value: bool) -> Self {
        self.props.insert("HintIgnore".into(), owned(Value::from(value)));
        self
    }

    pub fn hint_partitionable(mut self, value: bool) -> Self {
        self.props
            .insert("HintPartitionable".into(), owned(Value::from(value)));
        self
    }

    pub fn read_only(mut self, value: bool) -> Self {
        self.props.insert("ReadOnly".into(), owned(Value::from(value)));
        self
    }

    pub fn device_instead_of_preferred(mut self, dev: &str) -> Self {
        self.props.remove("PreferredDevice");
        self.props.insert("Device".into(), device_bytes(dev));
        self
    }

    pub fn insert_into(self, managed: &mut ManagedObjects, block_name: &str) {
        let mut ifaces = HashMap::new();
        ifaces.insert(BLOCK.into(), self.props);
        if self.is_partition {
            ifaces.insert(PARTITION.into(), HashMap::new());
        }
        managed.insert(block_path(block_name), ifaces);
    }
}

pub struct DriveBuilder {
    removable: bool,
    vendor: String,
    model: String,
}

impl DriveBuilder {
    pub fn removable(_name: &str) -> Self {
        Self {
            removable: true,
            vendor: "Kingston".into(),
            model: "DataTraveler 3.0".into(),
        }
    }

    pub fn fixed(name: &str) -> Self {
        let _ = name;
        Self {
            removable: false,
            vendor: "WDC".into(),
            model: "Internal".into(),
        }
    }

    pub fn insert_into(self, managed: &mut ManagedObjects, drive_name: &str) {
        let mut props = HashMap::new();
        props.insert("Removable".into(), owned(Value::from(self.removable)));
        props.insert("Vendor".into(), owned(Value::from(self.vendor.as_str())));
        props.insert("Model".into(), owned(Value::from(self.model.as_str())));
        let mut ifaces = HashMap::new();
        ifaces.insert(DRIVE_IF.into(), props);
        managed.insert(drive_path(drive_name), ifaces);
    }
}

/// Pendrive removível `sde` + partição `sde1` (só o disco inteiro deve listar).
pub fn removable_disk_with_partition() -> ManagedObjects {
    let mut managed = ManagedObjects::new();
    DriveBuilder::removable("Kingston_xxx").insert_into(&mut managed, "Kingston_xxx");
    BlockBuilder::whole_disk("sde", "Kingston_xxx").insert_into(&mut managed, "sde");
    BlockBuilder::partition("sde1", "Kingston_xxx").insert_into(&mut managed, "sde1");
    managed
}

/// Dois pendrives removíveis para testar ordenação.
pub fn two_removable_disks() -> ManagedObjects {
    let mut managed = ManagedObjects::new();
    DriveBuilder::removable("A").insert_into(&mut managed, "DriveA");
    DriveBuilder::removable("B").insert_into(&mut managed, "DriveB");
    BlockBuilder::whole_disk("sdf", "DriveB").insert_into(&mut managed, "sdf");
    BlockBuilder::whole_disk("sde", "DriveA").insert_into(&mut managed, "sde");
    managed
}

/// Disco removível marcado como HintSystem (não deve listar).
pub fn hint_system_removable() -> ManagedObjects {
    let mut managed = ManagedObjects::new();
    DriveBuilder::removable("Sys").insert_into(&mut managed, "Sys");
    BlockBuilder::whole_disk("sr0", "Sys")
        .hint_system(true)
        .insert_into(&mut managed, "sr0");
    managed
}

/// Disco removível com HintIgnore.
pub fn hint_ignore_removable() -> ManagedObjects {
    let mut managed = ManagedObjects::new();
    DriveBuilder::removable("Ign").insert_into(&mut managed, "Ign");
    BlockBuilder::whole_disk("sdc", "Ign")
        .hint_ignore(true)
        .insert_into(&mut managed, "sdc");
    managed
}

/// Disco removível sem HintPartitionable.
pub fn not_partitionable_removable() -> ManagedObjects {
    let mut managed = ManagedObjects::new();
    DriveBuilder::removable("NP").insert_into(&mut managed, "NP");
    BlockBuilder::whole_disk("sdd", "NP")
        .hint_partitionable(false)
        .insert_into(&mut managed, "sdd");
    managed
}

/// Disco removível somente leitura.
pub fn read_only_removable() -> ManagedObjects {
    let mut managed = ManagedObjects::new();
    DriveBuilder::removable("RO").insert_into(&mut managed, "RO");
    BlockBuilder::whole_disk("sdb", "RO")
        .read_only(true)
        .insert_into(&mut managed, "sdb");
    managed
}

/// Disco removível usando propriedade Device em vez de PreferredDevice.
pub fn removable_with_device_property() -> ManagedObjects {
    let mut managed = ManagedObjects::new();
    DriveBuilder::removable("Dev").insert_into(&mut managed, "Dev");
    BlockBuilder::whole_disk("sdz", "Dev")
        .device_instead_of_preferred("/dev/sdz")
        .insert_into(&mut managed, "sdz");
    managed
}

/// Disco interno não removível (não deve listar).
pub fn fixed_internal_disk() -> ManagedObjects {
    let mut managed = ManagedObjects::new();
    DriveBuilder::fixed("Internal_0").insert_into(&mut managed, "Internal_0");
    BlockBuilder::whole_disk("sda", "Internal_0").insert_into(&mut managed, "sda");
    managed
}
