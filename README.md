# FormatPen

Formatador de pendrive com interface gráfica GTK4/Libadwaita para Linux.

## Funcionalidades

- Lista dispositivos removíveis (pendrives) conectados — **o disco inteiro** (`/dev/sdd`), não partições individuais
- Seleção do dispositivo a formatar
- Escolha do tipo de sistema de arquivos: FAT32, exFAT, NTFS, ext4
- Definição do nome (label) do volume
- Formatação via UDisks2 (Polkit) — sem necessidade de rodar como root
- Reparticiona o disco inteiro em **uma única partição** (remove layouts multiboot: Ventoy, ISO híbrida, etc.)
- Diálogo de confirmação antes de formatar
- Desmontagem automática de todas as partições antes da formatação

## Requisitos

- Linux com UDisks2 e Polkit
- GTK4 e Libadwaita (interface gráfica)
- Ferramentas de formatação: `dosfstools`, `exfatprogs`, `ntfs-3g`, `e2fsprogs`, `parted`

### Pacotes de runtime (AppImage e uso geral)

| Componente | Fedora | Debian / Ubuntu |
|------------|--------|-----------------|
| GTK4 + Libadwaita | `gtk4`, `libadwaita` | `libgtk-4-1`, `libadwaita-1-0` |
| UDisks2 + Polkit | `udisks2`, `polkit` | `udisks2`, `policykit-1` |
| FAT32 | `dosfstools` | `dosfstools` |
| exFAT | `exfatprogs` | `exfatprogs` |
| NTFS | `ntfs-3g` | `ntfs-3g` |
| ext4 | `e2fsprogs` | `e2fsprogs` |
| Reparticionar | `parted` | `parted` |

```bash
# Fedora — runtime completo
sudo dnf install gtk4 libadwaita udisks2 polkit dosfstools exfatprogs ntfs-3g e2fsprogs parted

# Debian / Ubuntu — runtime completo
sudo apt install libgtk-4-1 libadwaita-1-0 udisks2 policykit-1 dosfstools exfatprogs ntfs-3g e2fsprogs parted
```

Se UDisks2 ou alguma ferramenta faltar, o app exibe um **banner** na abertura com instruções (comando adaptado à distro via `/etc/os-release`).

## Instalação das dependências de build (Fedora/Debian/Ubuntu)

```bash
# Rust (se ainda não instalado)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Dependências GTK4 e build (Fedora)
sudo dnf install gtk4-devel libadwaita-devel meson desktop-file-utils gcc

# Dependências GTK4 e build (Debian/Ubuntu)
sudo apt install libgtk-4-dev libadwaita-1-dev meson desktop-file-utils gcc

# Pacotes para formatação (geralmente já instalados)
# dosfstools (FAT32), exfatprogs (exFAT), ntfs-3g (NTFS), e2fsprogs (ext4), parted
```

## AppImage

O AppImage empacota só o binário e metadados (ícone, `.desktop`). Veja [Requisitos de runtime](#pacotes-de-runtime-appimage-e-uso-geral) acima.

Build local (Fedora):

```bash
sudo dnf install gtk4-devel libadwaita-devel gcc pkg-config fuse fuse-libs curl
chmod +x scripts/build-appimage.sh
./scripts/build-appimage.sh
./FormatPen-*-x86_64.AppImage
```

Em sistemas sem FUSE2 (ex.: Fedora Atomic 44+):

```bash
./FormatPen-*-x86_64.AppImage --appimage-extract-and-run
```

Releases com tag `v*` geram o AppImage automaticamente via GitHub Actions (`.github/workflows/appimage.yml`). Os requisitos de runtime estão na seção [Pacotes de runtime](#pacotes-de-runtime-appimage-e-uso-geral) acima.

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

## Testes

```bash
# Testes unitários (sem hardware, sem GTK em execução)
cargo test

# Testes de integração com UDisks2 (só listagem; não altera discos)
cargo test --test udisks_integration

# Formatação real em pendrive (APAGA DADOS — opcional)
# export FORMATPEN_TEST_DISK=/org/freedesktop/UDisks2/block_devices/sdd
# export FORMATPEN_TEST_DRIVE=/org/freedesktop/UDisks2/drives/...
# cargo test --test udisks_integration formata_disco_de_teste_ponta_a_ponta -- --ignored --nocapture
```

## Estrutura do projeto

```
FormatPen/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── label_validation.rs
│   ├── app.rs
│   ├── environment.rs
│   ├── window.rs
│   ├── models/
│   │   └── drive.rs
│   ├── services/
│   │   ├── udisks.rs
│   │   └── format.rs
│   ├── tests/          (integração UDisks2)
│   └── ui/
│       ├── drive_list.rs
│       └── format_form.rs
├── data/
│   ├── com.formatpen.FormatPen.desktop
│   └── icons/hicolor/scalable/apps/com.formatpen.FormatPen.svg
└── scripts/
    └── build-appimage.sh
```

## Licença

MIT
