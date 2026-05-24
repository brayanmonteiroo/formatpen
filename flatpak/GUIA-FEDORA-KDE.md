# FormatPen — Flatpak no Fedora (KDE)

KDE usa o app normalmente; o runtime Flatpak é **`org.gnome.Platform` 50** (GTK4/Libadwaita).

## Onde rodar comandos

| Onde | O quê |
|------|--------|
| Qualquer pasta | `sudo dnf install ...`, `flatpak install ...` |
| Raiz do repo `formatpen/` | `./scripts/build-flatpak.sh` |

## Build rápido

```bash
sudo dnf install flatpak flatpak-builder appstream
flatpak remote-add --if-not-exists flathub https://flathub.org/repo/flathub.flatpakrepo
flatpak install flathub org.gnome.Platform//50 org.gnome.Sdk//50
flatpak install flathub org.freedesktop.Sdk.Extension.rust-stable//25.08

cd ~/dev/github/formatpen
./scripts/build-flatpak.sh
flatpak run com.formatpen.FormatPen
```

## Problemas comuns

- **`No remote refs found for flathub`**: use Flathub no **sistema** (sem `--user` no `flatpak install` do SDK).
- **`index.crates.io`**: manifest usa `build-args: --share=network`.
- **GPG no `formatpen-local`**: o script usa `--no-gpg-verify` no remoto local.

Publicação na Flathub: **[FLATHUB.md](FLATHUB.md)**.
