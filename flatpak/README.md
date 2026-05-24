# FormatPen — Flatpak

## Pré-requisitos (Fedora)

```bash
sudo dnf install flatpak flatpak-builder appstream
flatpak remote-add --if-not-exists flathub https://flathub.org/repo/flathub.flatpakrepo
```

Instale o runtime e o SDK (padrão **GNOME 50**; para outra versão, defina `FLATPAK_RUNTIME_VERSION` antes do build):

```bash
flatpak install -y flathub org.gnome.Platform//50 org.gnome.Sdk//50
flatpak install -y flathub org.freedesktop.Sdk.Extension.rust-stable//25.08
```

## Build e instalação no usuário

Na **raiz do repositório**:

```bash
./scripts/build-flatpak.sh
```

Ou manualmente:

```bash
flatpak-builder --force-clean --user --install-deps-from=flathub \
  flatpak/.flatpak-builder flatpak/com.formatpen.FormatPen.yml

flatpak-builder --user --install-deps-from=flathub --repo=flatpak/repo \
  --force-clean flatpak/.flatpak-builder flatpak/com.formatpen.FormatPen.yml

flatpak --user remote-add --if-not-exists formatpen-local flatpak/repo
flatpak --user install -y formatpen-local com.formatpen.FormatPen
```

## Executar

```bash
flatpak run com.formatpen.FormatPen
```

## Validar metadados (Flathub)

```bash
appstreamcli validate --pedantic data/com.formatpen.FormatPen.metainfo.xml
```

## Publicar na Flathub

Guia completo (tag Git, `cargo-sources.json`, PR na Flathub): **[FLATHUB.md](FLATHUB.md)**.

Modelo de manifest para o repositório `flathub/com.formatpen.FormatPen`: [`com.formatpen.FormatPen.flathub.yml`](com.formatpen.FormatPen.flathub.yml).

## Permissões

O app fala com **UDisks2** no barramento de sistema para formatar discos. O host precisa ter `udisks2` e ferramentas de filesystem (`dosfstools`, `exfatprogs`, `ntfs-3g`, `e2fsprogs`).
