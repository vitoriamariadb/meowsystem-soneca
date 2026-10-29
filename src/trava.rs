use std::future::Future;
use std::sync::Arc;

pub type Resultado<T> = Result<T, String>;

pub const O_QUE: &str = "handle-lid-switch:sleep:idle";
pub const QUEM: &str = "MeowSystem Soneca";
pub const PORQUE: &str = "Modo Desperto";
pub const COMO: &str = "block";

#[derive(Debug)]
pub struct Segurada {
    _fd: zbus::zvariant::OwnedFd,
}

pub trait Logind: Clone + Send + Sync + 'static {
    fn inibir(&self) -> impl Future<Output = Resultado<Segurada>> + Send;
}

#[derive(Clone, Debug, Default)]
pub struct Sistema {
    conexao: Arc<tokio::sync::OnceCell<zbus::Connection>>,
}

const DESTINO: &str = "org.freedesktop.login1";
const CAMINHO: &str = "/org/freedesktop/login1";
const INTERFACE: &str = "org.freedesktop.login1.Manager";

fn erro(e: impl std::fmt::Display) -> String {
    e.to_string()
}

impl Sistema {
    #[cfg(test)]
    pub fn sobre(conexao: zbus::Connection) -> Self {
        let s = Self::default();
        s.conexao.set(conexao).expect("recém-criada");
        s
    }
}

impl Logind for Sistema {
    async fn inibir(&self) -> Resultado<Segurada> {
        let conexao = self.conexao.get_or_try_init(zbus::Connection::system).await.map_err(erro)?;
        let proxy = zbus::Proxy::new(conexao, DESTINO, CAMINHO, INTERFACE).await.map_err(erro)?;
        let fd = proxy
            .call::<_, _, zbus::zvariant::OwnedFd>("Inhibit", &(O_QUE, QUEM, PORQUE, COMO))
            .await
            .map_err(erro)?;
        Ok(Segurada { _fd: fd })
    }
}

pub async fn segurar<L: Logind>(logind: &L) -> Resultado<Segurada> {
    logind.inibir().await
}

#[cfg(test)]
mod testes {
    use super::*;
    use std::io::Read;
    use std::os::unix::net::UnixStream;
    use tokio::net::UnixStream as Fluxo;
    use std::sync::Mutex;
    use std::time::Duration;

    #[derive(Default)]
    struct LogindFalso {
        pedidos: Arc<Mutex<Vec<[String; 4]>>>,
        pontas: Arc<Mutex<Vec<UnixStream>>>,
        recusar: bool,
    }

    #[zbus::interface(name = "org.freedesktop.login1.Manager")]
    impl LogindFalso {
        fn inhibit(
            &self,
            what: String,
            who: String,
            why: String,
            mode: String,
        ) -> zbus::fdo::Result<zbus::zvariant::OwnedFd> {
            if self.recusar {
                return Err(zbus::fdo::Error::AccessDenied("sem permissão".into()));
            }
            self.pedidos.lock().unwrap().push([what, who, why, mode]);
            let (nossa, deles) = UnixStream::pair().map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;
            self.pontas.lock().unwrap().push(nossa);
            Ok(std::os::fd::OwnedFd::from(deles).into())
        }
    }

    struct Banca {
        sistema: Sistema,
        pedidos: Arc<Mutex<Vec<[String; 4]>>>,
        pontas: Arc<Mutex<Vec<UnixStream>>>,
        _servidor: zbus::Connection,
    }

    async fn banca(recusar: bool) -> Banca {
        let falso = LogindFalso { recusar, ..Default::default() };
        let pedidos = falso.pedidos.clone();
        let pontas = falso.pontas.clone();
        let (a, b) = Fluxo::pair().unwrap();
        let guid = zbus::Guid::generate();
        let servidor = tokio::spawn(async move {
            zbus::connection::Builder::unix_stream(a)
                .server(guid)
                .unwrap()
                .p2p()
                .serve_at(CAMINHO, falso)
                .unwrap()
                .build()
                .await
                .unwrap()
        });
        let cliente = zbus::connection::Builder::unix_stream(b).p2p().build().await.unwrap();
        Banca { sistema: Sistema::sobre(cliente), pedidos, pontas, _servidor: servidor.await.unwrap() }
    }

    fn rodar<F: Future>(f: F) -> F::Output {
        tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(f)
    }

    fn solta(ponta: &UnixStream) -> bool {
        ponta.set_read_timeout(Some(Duration::from_millis(50))).unwrap();
        let mut um = [0u8; 1];
        matches!((&*ponta).read(&mut um), Ok(0))
    }

    #[test]
    fn segurar_pede_a_trava_certa_ao_logind() {
        rodar(async {
            let b = banca(false).await;
            let trava = segurar(&b.sistema).await.unwrap();
            let pedidos = b.pedidos.lock().unwrap().clone();
            assert_eq!(pedidos.len(), 1);
            assert_eq!(
                pedidos[0],
                ["handle-lid-switch:sleep:idle", "MeowSystem Soneca", "Modo Desperto", "block"].map(String::from)
            );
            drop(trava);
        });
    }

    #[test]
    fn a_trava_dura_enquanto_o_applet_a_segura_e_solta_ao_largar() {
        rodar(async {
            let b = banca(false).await;
            let trava = segurar(&b.sistema).await.unwrap();
            let ponta = b.pontas.lock().unwrap().remove(0);
            assert!(!solta(&ponta), "a trava foi solta com o applet ainda segurando");
            drop(trava);
            assert!(solta(&ponta), "largar a trava não fechou o descritor do logind");
        });
    }

    #[test]
    fn uma_trava_por_vez_e_cada_uma_com_o_seu_descritor() {
        rodar(async {
            let b = banca(false).await;
            let primeira = segurar(&b.sistema).await.unwrap();
            let segunda = segurar(&b.sistema).await.unwrap();
            let pontas: Vec<_> = b.pontas.lock().unwrap().drain(..).collect();
            drop(primeira);
            assert!(solta(&pontas[0]));
            assert!(!solta(&pontas[1]));
            drop(segunda);
            assert!(solta(&pontas[1]));
        });
    }

    #[test]
    fn logind_que_recusa_vira_erro_sem_trava() {
        rodar(async {
            let b = banca(true).await;
            let e = segurar(&b.sistema).await.unwrap_err();
            assert!(e.contains("sem permissão"), "{e}");
            assert!(b.pedidos.lock().unwrap().is_empty());
        });
    }
}
