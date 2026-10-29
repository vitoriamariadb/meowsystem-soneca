use std::io::Write;
use std::path::{Path, PathBuf};

use crate::modelo::Modo;

const VERSAO: &str = "v1";
const ARQUIVO: &str = "modo";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Estado {
    arquivo: PathBuf,
}

impl Estado {
    pub fn sob(base: &Path, app_id: &str) -> Self {
        Self { arquivo: base.join("cosmic").join(app_id).join(VERSAO).join(ARQUIVO) }
    }

    pub fn da_casa(app_id: &str) -> Option<Self> {
        let base = std::env::var_os("XDG_STATE_HOME")
            .filter(|d| !d.is_empty())
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local").join("state")))?;
        Some(Self::sob(&base, app_id))
    }

    pub fn arquivo(&self) -> &Path {
        &self.arquivo
    }

    pub fn ler(&self) -> Modo {
        std::fs::read_to_string(&self.arquivo)
            .ok()
            .and_then(|t| Modo::da_chave(&t))
            .unwrap_or(Modo::PADRAO)
    }

    pub fn gravar(&self, modo: Modo) -> std::io::Result<()> {
        let pasta = self.arquivo.parent().ok_or(std::io::ErrorKind::InvalidInput)?;
        std::fs::create_dir_all(pasta)?;
        let provisorio = pasta.join(format!("{ARQUIVO}.{}.tmp", std::process::id()));
        let escrever = || -> std::io::Result<()> {
            let mut f = std::fs::File::create(&provisorio)?;
            writeln!(f, "{}", modo.chave())?;
            f.sync_all()?;
            std::fs::rename(&provisorio, &self.arquivo)
        };
        escrever().inspect_err(|_| {
            let _ = std::fs::remove_file(&provisorio);
        })
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    fn casa(nome: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("soneca-estado-{nome}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        d
    }

    #[test]
    fn sem_arquivo_vale_o_padrao() {
        let base = casa("vazio");
        assert_eq!(Estado::sob(&base, "a.b.C").ler(), Modo::PADRAO);
    }

    #[test]
    fn o_modo_gravado_volta_na_proxima_leitura() {
        let base = casa("volta");
        let e = Estado::sob(&base, "a.b.C");
        for m in [Modo::Desperto, Modo::Ronronar, Modo::Desperto] {
            e.gravar(m).unwrap();
            assert_eq!(Estado::sob(&base, "a.b.C").ler(), m);
        }
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn o_arquivo_mora_no_estado_do_cosmic_e_nao_deixa_provisorio() {
        let base = casa("lugar");
        let e = Estado::sob(&base, "io.github.meowsystem.Soneca");
        e.gravar(Modo::Desperto).unwrap();
        assert_eq!(e.arquivo(), base.join("cosmic/io.github.meowsystem.Soneca/v1/modo"));
        assert_eq!(std::fs::read_to_string(e.arquivo()).unwrap(), "desperto\n");
        let restos: Vec<_> = std::fs::read_dir(e.arquivo().parent().unwrap()).unwrap().flatten().collect();
        assert_eq!(restos.len(), 1);
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn arquivo_ilegivel_vale_o_padrao() {
        let base = casa("lixo");
        let e = Estado::sob(&base, "a.b.C");
        std::fs::create_dir_all(e.arquivo().parent().unwrap()).unwrap();
        std::fs::write(e.arquivo(), "talvez\n\u{0}").unwrap();
        assert_eq!(e.ler(), Modo::PADRAO);
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn gravar_sem_permissao_devolve_erro() {
        let base = casa("sem-pasta");
        std::fs::create_dir_all(&base).unwrap();
        std::fs::write(base.join("cosmic"), "arquivo no lugar da pasta").unwrap();
        assert!(Estado::sob(&base, "a.b.C").gravar(Modo::Desperto).is_err());
        let _ = std::fs::remove_dir_all(&base);
    }
}
