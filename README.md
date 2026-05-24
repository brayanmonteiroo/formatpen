# FormatPen

Formatador de pendrive com interface GTK4/Libadwaita para Linux.

## Funcionalidades

- Lista o **disco inteiro** (`/dev/sdd`), não partições soltas
- FAT32, exFAT, NTFS ou ext4; reparticiona em **uma única partição**
- Formatação via UDisks2 e Polkit (senha de admin quando pedido)

## Requisitos

- Linux com UDisks2
- Ferramentas de formatação no host (em geral já instaladas): `dosfstools`, `exfatprogs`, `ntfs-3g`, `e2fsprogs`

### Fedora

```bash
sudo dnf install gtk4-devel libadwaita-devel gcc pkg-config
# Flatpak local (opcional):
sudo dnf install flatpak flatpak-builder
```

### Debian / Ubuntu

```bash
sudo apt install libgtk-4-dev libadwaita-1-dev gcc pkg-config
```

### Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Desenvolvimento

Na raiz do repositório clonado:

```bash
cargo run
cargo test
cargo build --release
./target/release/formatpen
```

Teste de integração com UDisks2 (só listagem):

```bash
cargo test --test udisks_integration
```

## Flatpak (build local)

Manifest: [`flatpak/com.formatpen.FormatPen.yml`](flatpak/com.formatpen.FormatPen.yml) (código do diretório atual).

```bash
flatpak remote-add --if-not-exists flathub https://flathub.org/repo/flathub.flatpakrepo
flatpak install flathub org.gnome.Platform//50 org.gnome.Sdk//50
flatpak install flathub org.freedesktop.Sdk.Extension.rust-stable//25.08

./scripts/build-flatpak.sh
flatpak run com.formatpen.FormatPen
```

Artefatos de build (`flatpak/repo`, `flatpak/.flatpak-builder`) ficam no `.gitignore`.

## Flathub

Publicação via **pull request** em [flathub/flathub](https://github.com/flathub/flathub), branch base **`new-pr`** — ver [Submission](https://docs.flathub.org/docs/for-app-authors/submission).

Manifest de referência para o repo Flathub: [`flatpak/com.formatpen.FormatPen.flathub.yml`](flatpak/com.formatpen.FormatPen.flathub.yml) (git tag + `cargo-sources.json` gerado com [flatpak-cargo-generator](https://github.com/flatpak/flatpak-builder-tools/tree/master/cargo)).

Após aceito na Flathub, atualizações de pacote vão para `https://github.com/flathub/com.formatpen.FormatPen`.

### Nova versão (mantenedor)

1. Bump `version` em `Cargo.toml` e `<release>` em `data/com.formatpen.FormatPen.metainfo.xml`
2. Commit, tag (`v1.0.x`), push
3. Regenerar `cargo-sources.json` se `Cargo.lock` mudou:

```bash
python3 /caminho/flatpak-builder-tools/cargo/flatpak-cargo-generator.py \
  Cargo.lock -o cargo-sources.json
```

4. PR no repo `flathub/com.formatpen.FormatPen` com tag, commit e `cargo-sources.json` atualizados

## Estrutura

```
formatpen/
├── src/                 # aplicação Rust + GTK
├── data/                # desktop, metainfo, ícone
├── flatpak/             # manifests e screenshot (Flathub)
├── scripts/build-flatpak.sh
└── Cargo.toml
```

## Licença

MIT
