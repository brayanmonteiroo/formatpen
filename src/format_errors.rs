#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormatUserMessage {
    pub message: String,
    pub show_install_hint: bool,
}

/// Converte erros de formatação (UDisks2 / helpers) em mensagem amigável para o usuário.
pub fn user_message_for_format_error(error: &anyhow::Error) -> FormatUserMessage {
    let chain = format!("{error:#}").to_lowercase();

    if chain.contains("label for exfat filesystem is too long") {
        return FormatUserMessage {
            message: "Nome longo demais para exFAT. Use no máximo 11 caracteres ou deixe vazio."
                .to_string(),
            show_install_hint: false,
        };
    }

    if (chain.contains("label for vfat") && chain.contains("too long"))
        || (chain.contains("label for fat") && chain.contains("too long"))
    {
        return FormatUserMessage {
            message: "Nome longo demais para FAT32. Use no máximo 11 caracteres ou deixe vazio."
                .to_string(),
            show_install_hint: false,
        };
    }

    if chain.contains("not authorized") || chain.contains("notauthorized") {
        return FormatUserMessage {
            message: "Permissão negada. Confirme a senha de administrador quando o sistema pedir."
                .to_string(),
            show_install_hint: false,
        };
    }

    if chain.contains("busy")
        || chain.contains("mounted")
        || chain.contains("in use")
        || chain.contains("is busy")
    {
        return FormatUserMessage {
            message: "Pendrive em uso. Feche gerenciadores de arquivos, o Gerenciador de Partições ou outros programas que estejam acessando o dispositivo e tente de novo.".to_string(),
            show_install_hint: false,
        };
    }

    if chain.contains("no such file")
        || chain.contains("command not found")
        || chain.contains("failed to execute")
        || chain.contains("mkfs.")
        || chain.contains("mkfs ")
    {
        return FormatUserMessage {
            message: "Ferramenta de formatação ausente no sistema.".to_string(),
            show_install_hint: true,
        };
    }

    FormatUserMessage {
        message: "Não foi possível formatar o dispositivo. Tente de novo ou escolha outro tipo de sistema de arquivos.".to_string(),
        show_install_hint: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn err_from(msg: &str) -> anyhow::Error {
        anyhow::anyhow!("{msg}")
    }

    fn err_with_context(msg: &str, ctx: &str) -> anyhow::Error {
        anyhow::anyhow!("{ctx}: {msg}")
    }

    #[test]
    fn mapeia_label_exfat_longa() {
        let e = err_with_context(
            "org.freedesktop.UDisks2.Error.Failed: Label for exFAT filesystem is too long",
            "Falha ao criar partição e formatar",
        );
        let msg = user_message_for_format_error(&e);
        assert!(!msg.show_install_hint);
        assert!(msg.message.contains("11 caracteres"));
    }

    #[test]
    fn mapeia_permissao_negada() {
        let e = err_from("org.freedesktop.UDisks2.Error.NotAuthorized: Not authorized");
        let msg = user_message_for_format_error(&e);
        assert!(!msg.show_install_hint);
        assert!(msg.message.contains("Permissão negada"));
    }

    #[test]
    fn mapeia_dispositivo_ocupado() {
        let e = err_from("Device is busy");
        let msg = user_message_for_format_error(&e);
        assert!(!msg.show_install_hint);
        assert!(msg.message.contains("em uso"));
        assert!(msg.message.contains("gerenciadores de arquivos"));
        assert!(!msg.message.to_lowercase().contains("dolphin"));
    }

    #[test]
    fn mapeia_ferramenta_ausente() {
        let e = err_from("Failed to execute mkfs.vfat: No such file or directory");
        let msg = user_message_for_format_error(&e);
        assert!(msg.show_install_hint);
        assert!(msg.message.contains("Ferramenta"));
    }

    #[test]
    fn mapeia_label_vfat_longa() {
        let e = err_with_context(
            "org.freedesktop.UDisks2.Error.Failed: Label for vfat filesystem is too long",
            "Falha ao criar partição e formatar",
        );
        let msg = user_message_for_format_error(&e);
        assert!(!msg.show_install_hint);
        assert!(msg.message.contains("FAT32"));
        assert!(msg.message.contains("11 caracteres"));
    }

    #[test]
    fn erro_desconhecido_sem_dica_instalacao() {
        let e = err_from("Something unexpected happened");
        let msg = user_message_for_format_error(&e);
        assert!(!msg.show_install_hint);
        assert!(msg.message.contains("Não foi possível formatar"));
    }
}
