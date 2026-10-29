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

O `Cargo.lock` é o do meowsystem-performance mais este pacote. O único acréscimo é a aresta
`zbus` para `getrandom 0.4.3`, que vem do recurso `p2p` e só entra nos testes (o logind falso
conversa por um par de sockets, sem barramento).

## O que o applet lê e escreve

| | lê | escreve |
|---|---|---|
| `org.freedesktop.login1` (D-Bus do sistema) | nada | `Inhibit`, só com o modo Desperto |
| `/sys/class/power_supply/*/{type,scope,capacity,status}` | a bateria do notebook | nada |
| `org.freedesktop.Notifications` (D-Bus da sessão) | nada | uma notificação quando a bateria força o Ronronar |
| `~/.local/state/cosmic/io.github.meowsystem.Soneca/v1/modo` | o modo | o modo, por clique, pela linha de comando e na troca por bateria |

O arquivo do modo tem uma linha, `desperto` ou `ronronar`. O applet relê a cada 2 s, e é assim
que `--desperto` e `--ronronar` valem com o applet no ar.

## Quem continua fazendo o quê

- Quem mexe na trava é só o applet. Nada do MeowSystem segura tampa nem ocioso.
- Applet fora do ar: nenhuma trava, o sistema volta a mandar. O modo gravado continua lá e vale
  de novo quando o applet sobe.

## Tema

O contrato dos outros applets do MeowSystem. No pino: `TEMA="io.github.meowsystem.Soneca"` e
`TEMA_CHAVES="acento:@acento destaque:base fundo:base borda:surface1 texto:text bloco:surface0"`.
Sem o arquivo, vale o tema do COSMIC.
