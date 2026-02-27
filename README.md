# FormatPen

Formatador de pendrive com interface gráfica GTK4/Libadwaita para Linux.

## Funcionalidades

- Lista dispositivos removíveis (pendrives) conectados
- Seleção do dispositivo a formatar
- Escolha do tipo de sistema de arquivos: FAT32, exFAT, NTFS, ext4
- Definição do nome (label) do volume
- Formatação via UDisks2 (Polkit) - sem necessidade de rodar como root
- Diálogo de confirmação antes de formatar
- Desmontagem automática antes da formatação

## Requisitos

- Linux com UDisks2
- Fedora: GTK4, Libadwaita e ferramentas de build

## Instalação das dependências Rust e GTK4 (Fedora/Debian/Ubuntu)

```bash
# Rust (se ainda não instalado)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Dependências GTK4 e build (Fedora)
sudo dnf install gtk4-devel libadwaita-devel meson desktop-file-utils gcc

# Dependências GTK4 e build (Debian/Ubuntu)
sudo apt install libgtk-4-dev libadwaita-1-dev meson desktop-file-utils gcc

# Pacotes para formatação (geralmente já instalados)
# dosfstools (FAT32), exfat-utils (exFAT), ntfs-3g (NTFS), e2fsprogs (ext4)
```

## Compilação e execução

```bash
# Desenvolvimento (modo debug)
cargo run

# Build de produção
cargo build --release

# Executar o binário compilado
./target/release/formatpen

# Ou instalar em ~/.cargo/bin
cargo install --path .
formatpen
```

## Estrutura do projeto

```
FormatPen/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── app.rs
│   ├── window.rs
│   ├── models/
│   │   └── drive.rs
│   ├── services/
│   │   ├── udisks.rs
│   │   └── format.rs
│   └── ui/
│       ├── drive_list.rs
│       └── format_form.rs
└── data/
    └── com.formatpen.FormatPen.desktop
```

## Licença

MIT
