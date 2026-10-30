# Integração com o MeowSystem

O Soneca é um programa próprio, sem upstream. O MeowSystem o consome pelo pino
`assets/extensoes/soneca.pino`, como os outros applets da casa: clona este repositório num
commit fixo, compila com o libcosmic do pino e instala o binário, o `.desktop` e o ícone.

## Onde está

| | |
|---|---|
| repositório | <https://github.com/vitoriamariadb/meowsystem-soneca> |
| clone (SSH desta máquina) | `git@github.com-personal:vitoriamariadb/meowsystem-soneca.git` |
| app id | `io.github.meowsystem.Soneca` |
| binário | `meowsystem-soneca` (`--estado`, `--desperto`, `--ronronar`) |
| `.desktop` | `data/io.github.meowsystem.Soneca.desktop.in` (o `@bindir@` vira o diretório do binário) |
| ícone do tema | `data/icones/io.github.meowsystem.Soneca-symbolic.svg`, em `hicolor/scalable/apps` |
| libcosmic | `1f6dc991eaa5115a09be2505cc8a323e5b2b0bff`, o mesmo do meowsystem-performance |

## Build

```bash
cargo build --release --locked      # target/release/meowsystem-soneca
cargo test --release --locked
```

O `Cargo.lock` é o do meowsystem-performance mais este pacote, sem crate nova. Os acréscimos são
duas arestas: `notify` (o vigia do arquivo do modo, a mesma versão que o `cosmic-config` já usa) e
`zbus` para `getrandom 0.4.3`, que vem do recurso `p2p` e só entra nos testes (o logind falso
conversa por um par de sockets, sem barramento).

## O que o applet lê e escreve

| | lê | escreve |
|---|---|---|
| `org.freedesktop.login1` (D-Bus do sistema) | nada | `Inhibit`, só com o modo Desperto |
| `/sys/class/power_supply/*/{type,scope,capacity,status}` | a bateria do notebook | nada |
| `org.freedesktop.Notifications` (D-Bus da sessão) | nada | uma notificação quando a bateria força o Ronronar |
| `~/.local/state/cosmic/io.github.meowsystem.Soneca/v1/modo` | o modo | o modo, por clique, pela linha de comando e na troca por bateria |

O arquivo do modo tem uma linha, `desperto` ou `ronronar`. O applet vigia a pasta dele com
inotify, e é assim que `--desperto` e `--ronronar` valem na hora com o applet no ar. No Ronronar
nada acorda o applet; no Desperto ele confere a bateria a cada 30 s e, se o `logind` recusou a
trava, pede de novo.

O ícone e o marcado do popup mostram Desperto só com a trava na mão: se o `logind` recusa, o
painel mostra Ronronar e o popup diz que o sistema recusou.

## Quem continua fazendo o quê

- Quem mexe na trava é só o applet. Nada do MeowSystem segura tampa nem ocioso.
- Applet fora do ar: nenhuma trava, o sistema volta a mandar. O modo gravado continua lá e vale
  de novo quando o applet sobe.

## Tema

O contrato dos outros applets do MeowSystem. No pino: `TEMA="io.github.meowsystem.Soneca"` e
`TEMA_CHAVES="acento:@acento destaque:base fundo:base borda:surface1 texto:text bloco:surface0"`.
Sem o arquivo, vale o tema do COSMIC.
