//! Testes que exigem UDisks2 no sistema (Linux + D-Bus).
//!
//! Executar com: cargo test --test udisks_integration -- --ignored --nocapture

use formatpen::services::{FormatService, FilesystemType, UDisksService};

#[test]
fn lista_discos_removiveis_via_udisks() {
    let drives = UDisksService::list_removable_drives().expect("UDisks2 deve estar disponível");
    for drive in &drives {
        assert!(drive.object_path.contains("block_devices/"));
        assert!(drive.device_path.starts_with("/dev/"));
        assert!(drive.is_removable);
    }
}

#[test]
#[ignore = "apaga dados do dispositivo; defina FORMATPEN_TEST_DISK e FORMATPEN_TEST_DRIVE"]
fn formata_disco_de_teste_ponta_a_ponta() {
    let disk = std::env::var("FORMATPEN_TEST_DISK")
        .expect("FORMATPEN_TEST_DISK=/org/freedesktop/UDisks2/block_devices/sdd");
    let drive = std::env::var("FORMATPEN_TEST_DRIVE").expect(
        "FORMATPEN_TEST_DRIVE=/org/freedesktop/UDisks2/drives/Kingston_...",
    );

    FormatService::format(&disk, &drive, FilesystemType::ExFat, Some("FormatPenTest"))
        .expect("formatação de teste deve concluir");
}
