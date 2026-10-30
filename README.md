# meowsystem-soneca

O applet **Soneca** na topbar do COSMIC: dois modos para o que o PC faz quando a tampa fecha ou
a máquina fica parada.

- **Desperto**: o PC não suspende com a tampa fechada nem por ociosidade. O applet segura uma
  trava do `logind` (`handle-lid-switch:sleep:idle`, modo `block`) enquanto o modo está ligado.
- **Ronronar**: sem trava. O PC dorme como o sistema manda.

Faz parte do [MeowSystem](https://github.com/vitoriamariadb/MeowSystem) e usa a mesma base do
[meowsystem-performance](https://github.com/vitoriamariadb/meowsystem-performance): o mesmo
libcosmic pinado, a mesma estrutura e o mesmo tema.

## O que ele faz

- Na barra: o gato de olhos abertos (Desperto) ou o gato enrolado de olhos fechados (Ronronar).
  Passar o mouse mostra o nome do modo.
- No popup: os dois modos; o marcado usa o acento do pack. Um clique troca na hora.
- O modo escolhido fica guardado e volta no login seguinte. Sem escolha nenhuma, vale o Ronronar.
- O gato acordado aparece só com a trava de pé: se o sistema recusa, a barra mostra o Ronronar.
- No Desperto, na bateria e abaixo de 10% sem carregar, o applet solta a trava, volta para o
  Ronronar e avisa numa notificação curta.

```bash
meowsystem-soneca --estado     # imprime desperto ou ronronar e sai
meowsystem-soneca --desperto   # grava o modo; o applet no ar acompanha na hora
meowsystem-soneca --ronronar
```

## Como ele fala com o sistema

| o quê | por onde |
|---|---|
| a trava | D-Bus do sistema, `org.freedesktop.login1.Manager.Inhibit`, solta ao fechar o descritor |
| a bateria | `/sys/class/power_supply/*/{type,scope,capacity,status}` (só leitura) |
| o aviso | D-Bus da sessão, `org.freedesktop.Notifications.Notify` |
| o modo | `~/.local/state/cosmic/io.github.meowsystem.Soneca/v1/modo` |

Sem `sudo` e sem senha: a trava do `logind` é liberada para quem está na sessão ativa. Fechar o
applet, ou ele cair, solta a trava sozinha.

## Compilar e instalar

Precisa de Rust 1.93 ou mais novo e das bibliotecas de desenvolvimento do libcosmic
(`libxkbcommon-dev`, `libwayland-dev`, `pkg-config`).

```bash
cargo build --release --locked
cargo test --release --locked   # o logind de verdade nunca é chamado: os testes usam um falso
./install.sh                    # binário em ~/.local/bin, .desktop e ícone em ~/.local/share
./install.sh --remover          # desfaz
```

Depois, acrescente **Soneca** em Configurações > Área de trabalho > Painel > Applets.

## Tema

O mesmo contrato dos outros applets do MeowSystem: o applet lê
`~/.config/meowsystem/extensoes/io.github.meowsystem.Soneca/tema.conf` (chaves `acento`,
`fundo`, `borda`, `texto`, `destaque`, `bloco`) ao subir e a cada vez que o popup abre. Sem o
arquivo, vale o tema do COSMIC.

## Licença

GPL-3.0, a mesma do MeowSystem. Ver [`LICENSE`](LICENSE).
