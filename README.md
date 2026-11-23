# WordZ

O WordZ é um processador de texto para Windows e Linux, uma versão personalizada do
**[WordCraft](https://github.com/storytold/wordcraft) v0.18.1**, o processador de texto de
código aberto em Rust feito pelo time do ArtCraft.

Todo o editor (documentos .docx, estilos, tabelas, controle de alterações, referências, mala
direta, PDF) é o do WordCraft 0.18.1. O WordZ acrescenta as mudanças abaixo. O README original
do WordCraft está em [README-WordCraft.md](README-WordCraft.md).

> O WordZ não é afiliado ao ArtCraft nem ao WordCraft, nem endossado por eles. Também não tem
> relação com a Microsoft; "Word" e "Microsoft Word" são marcas da Microsoft, citadas aqui
> só para descrever formatos de arquivo (.docx).

## O que muda em relação ao WordCraft 0.18.1

### Impressão
- **Janela Imprimir (Ctrl+P)** no estilo clássico:
  - impressora com status, tipo, onde e comentário;
  - **Propriedades…**, que abre as preferências de impressão do driver (tipo de papel, qualidade);
  - saída inversa, imprimir para arquivo (PDF) e frente e verso;
  - intervalo de páginas (todas, atual, páginas, seleção) e páginas ímpares/pares;
  - cópias e agrupar;
  - páginas por folha, com a ordem (da esquerda para a direita, de cima para baixo ou repetir)
    e linhas entre as páginas;
  - ajuste ao tamanho do papel.
- A página Arquivo › Imprimir, com a visualização, continua disponível: em *Opções… ›
  Visualizar impressão*, ou desmarcando "Ctrl+P abre a janela Imprimir" em Opções › Avançado.
- Botão **Preferências de impressão** na página Arquivo › Imprimir.

### Imagens
- **Colar imagens copiadas de suítes de escritório** (WPS Office, entre outras), que põem um
  texto vazio junto com a imagem. Também reconhece imagens em JPEG e GIF na área de
  transferência do Windows.
- **Formato da Imagem › Tamanho**:
  - **Bloquear proporção**, que vale para todas as alças e para as caixas de largura e altura;
  - **Redefinir tamanho**;
  - alças de redimensionamento em todos os lados, também em imagens pequenas.

### Aparência
- Nome **WordZ** nas janelas, nos menus, nas mensagens e no instalador.
- Ícone próprio, original: uma folha com um "Z" sobre um fundo coral
  (`assets/app-icon/wordcraft.svg`).
- A janela Sobre informa que o WordZ é baseado no WordCraft 0.18.1, mantém o copyright do
  WordCraft e diz que o WordZ não é afiliado ao ArtCraft nem ao WordCraft.
- Publicador "WordZ" no instalador, nas propriedades do `.exe` e no pacote do Linux.
- Não procura nem instala atualizações: as atualizações do WordCraft substituiriam o WordZ.
- Sem o botão e os links do Discord.
- Traduções em português do Brasil dentro do projeto (`i18n-local/`), com os textos novos:
  compila em português sem baixar mais nada.

### Empacotamento
- `packaging/windows/msi-linux.sh`: gera um `.exe` e um instalador `.msi` simples a partir do
  Linux, com MinGW e wixl.

Por dentro, o programa mantém os nomes do WordCraft (o arquivo `wordcraft.exe` e as pastas de
configuração). Assim o WordZ instala por cima de um WordCraft e mantém as configurações.

## Como compilar

### Traduções

As traduções em português já vêm no projeto (`i18n-local/`) e entram sozinhas na compilação.
Para ter também os outros idiomas, baixe o craft-i18n ao lado da pasta do projeto:

```sh
git clone https://github.com/storytold/craft-i18n
git -C craft-i18n checkout $(cat wordcraft-main/craft-i18n.rev)
cp wordcraft-main/i18n-local/locales/pt-br/wordcraft.tsv craft-i18n/locales/pt-br/wordcraft.tsv
```

### Windows: instalador .msi

São necessários: Visual Studio Build Tools (C++ e Windows SDK), Rust, .NET SDK 8,
PowerShell 7 e o WiX 5:

```powershell
dotnet tool install --global wix --version 5.0.2
wix extension add -g WixToolset.UI.wixext/5.0.2
wix extension add -g WixToolset.Util.wixext/5.0.2
```

Depois, dentro desta pasta:

```powershell
$env:WORDCRAFT_BUILD_SHA = "local"
pwsh -ExecutionPolicy Bypass -File packaging/windows/package.ps1 -Arch x64
```

O `.msi` e o `.zip` portátil saem em `dist\release\`. Só o programa:
`cargo build --release -p wordcraft` gera `target\release\wordcraft.exe`.

### Linux: pacote .deb

```sh
WORDCRAFT_MAINTAINER="Seu Nome <seu@email>" bash packaging/linux/package.sh --formats deb
```

## Licença

O WordCraft é licenciado sob a [MIT](LICENSE-MIT) ou a [Apache 2.0](LICENSE-APACHE), à sua
escolha, e o WordZ segue as mesmas licenças. As fontes e outros materiais de terceiros
incluídos mantêm as próprias licenças, listadas em [NOTICE](NOTICE) e
[ATTRIBUTION.md](ATTRIBUTION.md).

Copyright do WordCraft: o time do ArtCraft e os colaboradores do WordCraft. As alterações do
WordZ estão nos arquivos listados abaixo, que foram modificados ou criados a partir do
WordCraft 0.18.1:

<details>
<summary>Arquivos alterados ou novos</summary>

```
README-WordCraft.md (o README original, renomeado)
README.md
apps/wordcraft/build.rs
apps/wordcraft/src/clipboard.rs
apps/wordcraft/src/main.rs
apps/wordcraft/src/print.rs
apps/wordcraft/src/printer_details.rs
apps/wordcraft/src/printing_preferences.rs
assets/app-icon/hicolor/128x128/apps/ai.storyteller.wordcraft.png
assets/app-icon/hicolor/16x16/apps/ai.storyteller.wordcraft.png
assets/app-icon/hicolor/24x24/apps/ai.storyteller.wordcraft.png
assets/app-icon/hicolor/256x256/apps/ai.storyteller.wordcraft.png
assets/app-icon/hicolor/32x32/apps/ai.storyteller.wordcraft.png
assets/app-icon/hicolor/48x48/apps/ai.storyteller.wordcraft.png
assets/app-icon/hicolor/512x512/apps/ai.storyteller.wordcraft.png
assets/app-icon/hicolor/64x64/apps/ai.storyteller.wordcraft.png
assets/app-icon/hicolor/scalable/apps/ai.storyteller.wordcraft.svg
assets/app-icon/wordcraft-1024.png
assets/app-icon/wordcraft-macos-512.png
assets/app-icon/wordcraft.ico
assets/app-icon/wordcraft.svg
crates/engine/src/cmd/file.rs
crates/engine/src/print.rs
crates/pdf/src/export.rs
crates/pdf/src/imposition.rs
crates/pdf/src/options.rs
crates/ui-egui/src/app/commands.rs
crates/ui-egui/src/backstage.rs
crates/ui-egui/src/chrome.rs
crates/ui-egui/src/dialogs/dialogs_layout.rs
crates/ui-egui/src/dialogs/dialogs_layout/print_dialog.rs
crates/ui-egui/src/dialogs/dispatch.rs
crates/ui-egui/src/frame.rs
crates/ui-egui/src/i18n.rs
crates/ui-egui/src/keytips.rs
crates/ui-egui/src/lib.rs
crates/ui-egui/src/objects.rs
crates/ui-egui/src/options.rs
crates/ui-egui/src/paste_picture.rs
crates/ui-egui/src/print_page.rs
crates/ui-egui/src/ribbon/picture.rs
crates/ui-egui/src/services.rs
crates/ui-egui/src/tests.rs
crates/ui-egui/src/updates_ui.rs
crates/ui-egui/src/window_ui.rs
crates/ui-egui/src/windows.rs
crates/ui-egui/tests/it/native_print.rs
i18n-local/README.md
i18n-local/locales/pt-br/common.tsv
i18n-local/locales/pt-br/wordcraft.tsv
packaging/linux/ai.storyteller.wordcraft.desktop
packaging/linux/ai.storyteller.wordcraft.metainfo.xml.in
packaging/linux/nfpm.yaml
packaging/linux/package.sh
packaging/windows/install-policy.json
packaging/windows/msi-linux.sh
packaging/windows/wordcraft-wixl.wxs
packaging/windows/wordcraft.wxs
```

</details>
