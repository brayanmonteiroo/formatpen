# Publicar o FormatPen na Flathub

Guia completo: tag Git **v1.0.0**, repositório de manifest na Flathub e revisão.

Documentação oficial: https://docs.flathub.org/docs/for-apps/publish-your-app

---

## Visão geral

A Flathub **não** usa o manifest da pasta `flatpak/` deste repositório diretamente. O fluxo é:

1. **Este repositório** (`formatpen`) — código-fonte, releases, tag `v1.0.0`.
2. **Repositório Flathub** (`flathub/com.formatpen.FormatPen`) — só manifest + `cargo-sources.json` (gerado), apontando para o GitHub com tag e commit fixos.

O build local com `type: dir` continua válido para você; na Flathub usa `type: git` + dependências Rust vendored (sem rede no sandbox).

---

## Parte 1 — Commit e tag no GitHub (este repositório)

### 1.1 Conferir versão

Tudo deve bater **1.0.0**:

| Arquivo | Campo |
|---------|--------|
| `Cargo.toml` | `version = "1.0.0"` |
| `data/com.formatpen.FormatPen.metainfo.xml` | `<release version="1.0.0" ...>` |

### 1.2 Commit na `main`

Na raiz do projeto:

```bash
cd ~/dev/github/formatpen
git status
git add -A
git commit -m "feat: empacotamento Flatpak 1.0.0 e guias de publicação"
```

O `Cargo.lock` **deve** entrar no commit (reprodutibilidade + exigência Flathub para apps Rust).

### 1.3 Criar tag anotada (recomendado)

```bash
git tag -a v1.0.0 -m "FormatPen 1.0.0 — primeira versão estável"
```

Conferir:

```bash
git show v1.0.0 --no-patch
git tag -l 'v*'
```

### 1.4 Enviar para o GitHub

```bash
git push origin main
git push origin v1.0.0
```

### 1.5 Anotar o commit da tag (obrigatório no manifest Flathub)

```bash
git rev-parse v1.0.0
```

Copie o hash completo (40 caracteres). Exemplo: `abc123...` — você colará no manifest da Flathub.

No GitHub: **Releases → Create a new release** → escolha a tag `v1.0.0`, título `1.0.0`, descreva as mudanças (opcional mas recomendado).

---

## Parte 2 — Requisitos Flathub (checklist)

Antes do pedido, confira:

- [ ] Código em repositório público: https://github.com/brayanmonteiroo/formatpen
- [ ] Licença no repositório (`LICENSE` — MIT)
- [ ] `Cargo.lock` commitado
- [ ] App ID estável: `com.formatpen.FormatPen` (desktop + metainfo + manifest)
- [ ] `.metainfo.xml` válido: `appstreamcli validate --pedantic data/com.formatpen.FormatPen.metainfo.xml`
- [ ] **Screenshots** no metainfo (Flathub exige) — ver Parte 4
- [ ] Ícone 128×128 ou SVG em `data/icons/...`
- [ ] App funciona instalado via Flatpak local (você já validou)

Permissões atuais (UDisks2 + Polkit + `filesystem=host`) são esperadas para formatador de disco; a revisão Flathub pode pedir justificativa por escrito no PR.

---

## Parte 3 — Gerar `cargo-sources.json`

A Flathub compila Rust **sem** baixar crates na hora. Gere o arquivo de fontes:

```bash
cd ~/dev/github/formatpen

# Ferramenta oficial (uma vez)
git clone --depth=1 https://github.com/flatpak/flatpak-builder-tools.git /tmp/flatpak-builder-tools

python3 /tmp/flatpak-builder-tools/cargo/flatpak-cargo-generator.py \
  Cargo.lock -o cargo-sources.json
```

O arquivo `cargo-sources.json` vai no **repositório Flathub**, não necessariamente neste (pode copiar na hora do PR).

Teste local do manifest Flathub (opcional):

```bash
flatpak-builder --force-clean --user build-dir flatpak/com.formatpen.FormatPen.flathub.yml
```

(Use o exemplo `com.formatpen.FormatPen.flathub.yml` depois de preencher `commit` e copiar `cargo-sources.json` para `flatpak/`.)

---

## Parte 4 — Screenshots no AppStream

Edite `data/com.formatpen.FormatPen.metainfo.xml` e adicione, por exemplo:

```xml
<screenshots>
  <screenshot type="default">
    <caption>Tela principal — seleção de dispositivo e sistema de arquivos</caption>
    <image>https://raw.githubusercontent.com/brayanmonteiroo/formatpen/v1.0.0/flatpak/screenshots/main.png</image>
  </screenshot>
</screenshots>
```

Passos práticos:

1. Tire um PNG da janela do FormatPen (a captura que você já tem serve).
2. Salve em `flatpak/screenshots/main.png` (crie a pasta).
3. Commit na `main`, inclua na tag `v1.0.0` **ou** faça tag `v1.0.1` só com screenshots se já tiver publicado v1.0.0.
4. Use URL `raw.githubusercontent.com` apontando para o commit/tag certo.

Revalide:

```bash
appstreamcli validate --pedantic data/com.formatpen.FormatPen.metainfo.xml
```

---

## Parte 5 — Repositório na Flathub

### 5.1 Pedido inicial

1. Leia: https://github.com/flathub/flathub/blob/master/CONTRIBUTING.md
2. Abra um issue em https://github.com/flathub/flathub (template de novo app), ou siga o fluxo atual em https://docs.flathub.org/docs/for-apps/submitting-an-app-new

Informe:

- **App ID:** `com.formatpen.FormatPen`
- **Upstream:** https://github.com/brayanmonteiroo/formatpen
- **Licença:** MIT
- **Por que `filesystem=host` e UDisks2:** formatar discos via serviço do sistema

### 5.2 Repositório do manifest

A equipe Flathub cria (ou você forka o template) o repositório:

`https://github.com/flathub/com.formatpen.FormatPen`

Estrutura mínima:

```
com.formatpen.FormatPen.yml   # manifest principal
cargo-sources.json            # gerado na Parte 3
```

Use o modelo em [`com.formatpen.FormatPen.flathub.yml`](com.formatpen.FormatPen.flathub.yml): substitua `COMMIT_DA_TAG_v1_0_0` pelo `git rev-parse v1.0.0`.

### 5.3 Pull request

1. Clone `flathub/com.formatpen.FormatPen` (quando existir).
2. Adicione `com.formatpen.FormatPen.yml` + `cargo-sources.json`.
3. Abra PR; o bot **flathubbot** dispara build de teste.
4. Responda comentários dos revisores (permissões, metainfo, runtime GNOME 50).

### 5.4 Atualizações futuras

Para **1.0.1**, **1.1.0**, etc.:

1. Bump `version` em `Cargo.toml` e `<release>` no metainfo.
2. Commit → tag `v1.0.1` → `git push origin v1.0.1`.
3. Regenerar `cargo-sources.json` se `Cargo.lock` mudou.
4. PR no repo Flathub atualizando `tag`, `commit` e `cargo-sources.json`.

A Flathub costuma usar branch `master` no repo de manifest; cada merge publica nova build.

---

## Parte 6 — Comandos resumidos (copiar e colar)

```bash
cd ~/dev/github/formatpen

# Validar metainfo
appstreamcli validate --pedantic data/com.formatpen.FormatPen.metainfo.xml

# Tag (se ainda não criou)
git tag -a v1.0.0 -m "FormatPen 1.0.0"
git push origin main
git push origin v1.0.0

# Hash para o manifest Flathub
git rev-parse v1.0.0

# Gerar fontes Cargo para Flathub
python3 /tmp/flatpak-builder-tools/cargo/flatpak-cargo-generator.py \
  Cargo.lock -o cargo-sources.json
```

---

## Links úteis

- [Requisitos de apps](https://docs.flathub.org/docs/for-apps/requirements)
- [Rust / Cargo](https://docs.flatpak.org/en/latest/dependencies.html#rust)
- [flatpak-builder-tools (cargo)](https://github.com/flatpak/flatpak-builder-tools/tree/master/cargo)
- [Novo pedido Flathub](https://github.com/flathub/flathub)
