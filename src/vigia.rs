use std::ffi::OsString;
use std::path::{Path, PathBuf};

use cosmic::iced::Subscription;
use cosmic::iced::futures::channel::mpsc;
use cosmic::iced::futures::{SinkExt, StreamExt};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};

pub fn vigiar(arquivo: &Path, avisar: impl Fn() + Send + 'static) -> notify::Result<RecommendedWatcher> {
    let pasta = arquivo.parent().ok_or_else(|| notify::Error::generic("arquivo sem pasta"))?;
    let nome: OsString = arquivo.file_name().ok_or_else(|| notify::Error::generic("arquivo sem nome"))?.into();
    std::fs::create_dir_all(pasta).map_err(notify::Error::io)?;
    let mut vigia = notify::recommended_watcher(move |evento: notify::Result<notify::Event>| {
        if let Ok(e) = evento
            && !e.kind.is_access()
            && e.paths.iter().any(|p| p.file_name() == Some(nome.as_os_str()))
        {
            avisar();
        }
    })?;
    vigia.watch(pasta, RecursiveMode::NonRecursive)?;
    Ok(vigia)
}

pub fn mudancas(arquivo: PathBuf) -> Subscription<()> {
    Subscription::run_with(arquivo, |arquivo| {
        let arquivo = arquivo.clone();
        cosmic::iced::stream::channel(4, async move |mut saida| {
            let (tx, mut rx) = mpsc::channel::<()>(1);
            let vigia = vigiar(&arquivo, move || {
                let _ = tx.clone().try_send(());
            });
            let _vigia = match vigia {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("meowsystem-soneca: {}: {e}", arquivo.display());
                    return std::future::pending().await;
                }
            };
            while rx.next().await.is_some() {
                if saida.send(()).await.is_err() {
                    break;
                }
            }
        })
    })
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::estado::Estado;
    use crate::modelo::Modo;
    use std::sync::mpsc as fila;
    use std::time::Duration;

    fn casa(nome: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("soneca-vigia-{nome}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        d
    }

    #[test]
    fn a_troca_pela_linha_de_comando_chega_sem_releitura_periodica() {
        let base = casa("troca");
        let estado = Estado::sob(&base, "a.b.C");
        let (tx, rx) = fila::channel();
        let _v = vigiar(estado.arquivo(), move || {
            let _ = tx.send(());
        })
        .unwrap();
        assert!(estado.arquivo().parent().unwrap().is_dir(), "a pasta do estado não foi criada");
        estado.gravar(Modo::Desperto).unwrap();
        assert!(rx.recv_timeout(Duration::from_secs(3)).is_ok(), "gravar o modo não acordou o vigia");
        while rx.recv_timeout(Duration::from_millis(200)).is_ok() {}
        estado.gravar(Modo::Ronronar).unwrap();
        assert!(rx.recv_timeout(Duration::from_secs(3)).is_ok(), "a segunda troca não acordou o vigia");
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn outro_arquivo_na_pasta_nao_acorda_o_applet() {
        let base = casa("outro");
        let estado = Estado::sob(&base, "a.b.C");
        let (tx, rx) = fila::channel();
        let _v = vigiar(estado.arquivo(), move || {
            let _ = tx.send(());
        })
        .unwrap();
        std::fs::write(estado.arquivo().with_file_name("outro"), "x").unwrap();
        assert!(rx.recv_timeout(Duration::from_millis(500)).is_err());
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn sem_poder_criar_a_pasta_o_vigia_devolve_erro() {
        let base = casa("bloqueada");
        std::fs::create_dir_all(&base).unwrap();
        std::fs::write(base.join("cosmic"), "arquivo no lugar da pasta").unwrap();
        assert!(vigiar(Estado::sob(&base, "a.b.C").arquivo(), || {}).is_err());
        let _ = std::fs::remove_dir_all(&base);
    }
}
