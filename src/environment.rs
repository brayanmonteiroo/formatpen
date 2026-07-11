use crate::models::Drive;
use crate::services::UDisksService;
use zbus::blocking::Connection;

const UDISKS2_SERVICE: &str = "org.freedesktop.UDisks2";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeIssueKind {
    DbusUnavailable,
    UDisksUnavailable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeIssue {
    pub kind: RuntimeIssueKind,
    pub title: String,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallHint {
    pub label: String,
    pub command: String,
}

#[derive(Debug, Clone)]
pub struct DriveRefreshOutcome {
    pub drives: Vec<Drive>,
    pub runtime_issue: Option<RuntimeIssue>,
    pub format_tools_warning: Option<String>,
}

struct FormatTool {
    label: &'static str,
    command: &'static str,
}

const FORMAT_TOOLS: &[FormatTool] = &[
    FormatTool {
        label: "FAT32",
        command: "mkfs.vfat",
    },
    FormatTool {
        label: "exFAT",
        command: "mkfs.exfat",
    },
    FormatTool {
        label: "NTFS",
        command: "mkfs.ntfs",
    },
    FormatTool {
        label: "ext4",
        command: "mkfs.ext4",
    },
    FormatTool {
        label: "reparticionamento",
        command: "parted",
    },
];

/// Pacotes de runtime para exibir na UI e nas releases.
pub fn install_hint_for_host() -> InstallHint {
    install_hint_for_distro(parse_host_distro_id().as_deref())
}

pub fn install_hint_for_distro(distro_id: Option<&str>) -> InstallHint {
    match distro_id {
        Some("fedora") => InstallHint {
            label: "Fedora".to_string(),
            command: "sudo dnf install gtk4 libadwaita udisks2 polkit dosfstools exfatprogs ntfs-3g e2fsprogs parted".to_string(),
        },
        Some("ubuntu") | Some("debian") | Some("pop") | Some("linuxmint") => InstallHint {
            label: "Debian/Ubuntu".to_string(),
            command: "sudo apt install libgtk-4-1 libadwaita-1-0 udisks2 policykit-1 dosfstools exfatprogs ntfs-3g e2fsprogs parted".to_string(),
        },
        Some(id) => InstallHint {
            label: id.to_string(),
            command: "Instale UDisks2, Polkit, GTK4, Libadwaita e as ferramentas dosfstools, exfatprogs, ntfs-3g, e2fsprogs e parted.".to_string(),
        },
        None => InstallHint {
            label: "Linux".to_string(),
            command: "Fedora: sudo dnf install gtk4 libadwaita udisks2 polkit dosfstools exfatprogs ntfs-3g e2fsprogs parted\n\
                      Debian/Ubuntu: sudo apt install libgtk-4-1 libadwaita-1-0 udisks2 policykit-1 dosfstools exfatprogs ntfs-3g e2fsprogs parted"
                .to_string(),
        },
    }
}

pub fn check_udisks_available() -> Result<(), RuntimeIssue> {
    let connection = match Connection::system() {
        Ok(c) => c,
        Err(_) => {
            return Err(RuntimeIssue {
                kind: RuntimeIssueKind::DbusUnavailable,
                title: "D-Bus do sistema indisponível".to_string(),
                detail: "O FormatPen precisa do D-Bus do sistema para falar com o UDisks2.".to_string(),
            });
        }
    };

    let has_owner: bool = connection
        .call_method(
            Some("org.freedesktop.DBus"),
            "/org/freedesktop/DBus",
            Some("org.freedesktop.DBus"),
            "NameHasOwner",
            &(UDISKS2_SERVICE,),
        )
        .and_then(|reply| reply.body().deserialize())
        .unwrap_or(false);

    if !has_owner {
        let hint = install_hint_for_host();
        return Err(RuntimeIssue {
            kind: RuntimeIssueKind::UDisksUnavailable,
            title: "UDisks2 não está disponível".to_string(),
            detail: format!(
                "Instale e inicie o UDisks2 (e Polkit) para listar e formatar discos.\n\n{}:\n{}",
                hint.label, hint.command
            ),
        });
    }

    Ok(())
}

pub fn missing_format_tool_labels() -> Vec<&'static str> {
    missing_format_tool_labels_with(command_exists)
}

pub(crate) fn missing_format_tool_labels_with<F>(command_exists: F) -> Vec<&'static str>
where
    F: Fn(&str) -> bool,
{
    FORMAT_TOOLS
        .iter()
        .filter(|tool| !command_exists(tool.command))
        .map(|tool| tool.label)
        .collect()
}

pub(crate) fn format_tools_warning_message_with<F>(command_exists: F) -> Option<String>
where
    F: Fn(&str) -> bool,
{
    let missing = missing_format_tool_labels_with(command_exists);
    if missing.is_empty() {
        return None;
    }

    let hint = install_hint_for_host();
    Some(format!(
        "Ferramentas de formatação ausentes ({}) — a listagem funciona, mas formatar pode falhar.\n{}:\n{}",
        missing.join(", "),
        hint.label,
        hint.command
    ))
}

pub fn refresh_drives() -> DriveRefreshOutcome {
    if let Err(issue) = check_udisks_available() {
        eprintln!("Ambiente indisponível: {} — {}", issue.title, issue.detail);
        return DriveRefreshOutcome {
            drives: Vec::new(),
            runtime_issue: Some(issue),
            format_tools_warning: None,
        };
    }

    let drives = match UDisksService::list_removable_drives() {
        Ok(drives) => drives,
        Err(e) => {
            eprintln!("Erro ao listar drives: {e:#}");
            let hint = install_hint_for_host();
            return DriveRefreshOutcome {
                drives: Vec::new(),
                runtime_issue: Some(RuntimeIssue {
                    kind: RuntimeIssueKind::UDisksUnavailable,
                    title: "Não foi possível acessar o UDisks2".to_string(),
                    detail: format!(
                        "{e:#}\n\n{}:\n{}",
                        hint.label, hint.command
                    ),
                }),
                format_tools_warning: None,
            };
        }
    };

    let format_tools_warning = format_tools_warning_message();

    DriveRefreshOutcome {
        drives,
        runtime_issue: None,
        format_tools_warning,
    }
}

fn format_tools_warning_message() -> Option<String> {
    format_tools_warning_message_with(command_exists)
}

fn command_exists(command: &str) -> bool {
    std::process::Command::new("sh")
        .arg("-c")
        .arg(format!("command -v {command} >/dev/null 2>&1"))
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn parse_host_distro_id() -> Option<String> {
    parse_distro_id_from_os_release(std::fs::read_to_string("/etc/os-release").ok()?.as_str())
}

fn parse_distro_id_from_os_release(content: &str) -> Option<String> {
    for line in content.lines() {
        if let Some(id) = line.strip_prefix("ID=") {
            return Some(id.trim_matches('"').to_lowercase());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_distro_id_fedora() {
        let content = r#"NAME="Fedora Linux"
ID=fedora
"#;
        assert_eq!(parse_distro_id_from_os_release(content).as_deref(), Some("fedora"));
    }

    #[test]
    fn parse_distro_id_ubuntu_com_aspas() {
        let content = r#"ID="ubuntu"
"#;
        assert_eq!(parse_distro_id_from_os_release(content).as_deref(), Some("ubuntu"));
    }

    #[test]
    fn hint_fedora_contem_udisks2() {
        let hint = install_hint_for_distro(Some("fedora"));
        assert!(hint.command.contains("udisks2"));
        assert!(hint.command.contains("dnf"));
    }

    #[test]
    fn hint_debian_contem_policykit() {
        let hint = install_hint_for_distro(Some("debian"));
        assert!(hint.command.contains("policykit-1"));
        assert!(hint.command.contains("apt"));
    }

    #[test]
    fn aviso_ferramentas_ausentes_com_mock() {
        let msg = format_tools_warning_message_with(|cmd| cmd != "mkfs.exfat");
        let warning = msg.expect("deve avisar sobre ferramentas ausentes");
        assert!(warning.contains("exFAT"));
        assert!(warning.contains("Ferramentas de formatação ausentes"));
    }

    #[test]
    fn mock_sem_ferramentas_ausentes_retorna_none() {
        assert!(format_tools_warning_message_with(|_| true).is_none());
    }

    #[test]
    fn missing_format_tools_detecta_varias() {
        let missing = missing_format_tool_labels_with(|cmd| {
            matches!(cmd, "mkfs.vfat" | "parted")
        });
        assert!(missing.contains(&"exFAT"));
        assert!(missing.contains(&"NTFS"));
        assert!(!missing.contains(&"FAT32"));
    }

    #[test]
    fn refresh_com_udisks_indisponivel_retorna_erro() {
        if std::env::var("FORMATPEN_TEST_UDISKS_AVAILABLE").as_deref() == Ok("1") {
            return;
        }
        if check_udisks_available().is_ok() {
            let outcome = refresh_drives();
            assert!(outcome.runtime_issue.is_none());
            return;
        }
        let outcome = refresh_drives();
        assert!(outcome.runtime_issue.is_some());
        assert!(outcome.drives.is_empty());
    }
}
