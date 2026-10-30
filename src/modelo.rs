pub const LIMIAR_BATERIA: u8 = 10;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Modo {
    Desperto,
    Ronronar,
}

impl Modo {
    pub const TODOS: [Modo; 2] = [Modo::Desperto, Modo::Ronronar];
    pub const PADRAO: Modo = Modo::Ronronar;

    pub fn nome(self) -> &'static str {
        match self {
            Self::Desperto => "Desperto",
            Self::Ronronar => "Ronronar",
        }
    }

    pub fn chave(self) -> &'static str {
        match self {
            Self::Desperto => "desperto",
            Self::Ronronar => "ronronar",
        }
    }

    pub fn da_chave(texto: &str) -> Option<Self> {
        Self::TODOS.into_iter().find(|m| m.chave() == texto.trim())
    }
}

pub fn mostrado(escolhido: Modo, segurando: bool) -> Modo {
    if escolhido == Modo::Desperto && segurando { Modo::Desperto } else { Modo::Ronronar }
}

pub fn clicavel(opcao: Modo, escolhido: Modo, segurando: bool) -> bool {
    opcao != escolhido || opcao != mostrado(escolhido, segurando)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bateria {
    pub percentual: u8,
    pub descarregando: bool,
}

pub fn bateria_critica(bateria: Option<Bateria>) -> bool {
    bateria.is_some_and(|b| b.descarregando && b.percentual < LIMIAR_BATERIA)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Reconciliacao {
    pub modo: Modo,
    pub por_bateria: bool,
}

pub fn reconciliar(no_disco: Modo, bateria: Option<Bateria>) -> Reconciliacao {
    if no_disco == Modo::Desperto && bateria_critica(bateria) {
        return Reconciliacao { modo: Modo::Ronronar, por_bateria: true };
    }
    Reconciliacao { modo: no_disco, por_bateria: false }
}

#[cfg(test)]
mod testes {
    use super::*;

    fn bateria(percentual: u8, descarregando: bool) -> Option<Bateria> {
        Some(Bateria { percentual, descarregando })
    }

    #[test]
    fn os_modos_vao_e_voltam_pela_chave() {
        for m in Modo::TODOS {
            assert_eq!(Modo::da_chave(m.chave()), Some(m));
        }
        assert_eq!(Modo::da_chave(" desperto\n"), Some(Modo::Desperto));
        assert_eq!(Modo::da_chave("Desperto"), None);
        assert_eq!(Modo::da_chave(""), None);
    }

    #[test]
    fn o_padrao_deixa_o_pc_dormir_como_o_sistema_manda() {
        assert_eq!(Modo::PADRAO, Modo::Ronronar);
    }

    #[test]
    fn so_mostra_desperto_com_a_trava_na_mao() {
        assert_eq!(mostrado(Modo::Desperto, true), Modo::Desperto);
        assert_eq!(mostrado(Modo::Desperto, false), Modo::Ronronar);
        assert_eq!(mostrado(Modo::Ronronar, false), Modo::Ronronar);
        assert_eq!(mostrado(Modo::Ronronar, true), Modo::Ronronar);
    }

    #[test]
    fn com_o_desperto_recusado_os_dois_ficam_clicaveis() {
        assert!(clicavel(Modo::Desperto, Modo::Desperto, false));
        assert!(clicavel(Modo::Ronronar, Modo::Desperto, false));
        assert!(!clicavel(Modo::Desperto, Modo::Desperto, true));
        assert!(clicavel(Modo::Ronronar, Modo::Desperto, true));
        assert!(!clicavel(Modo::Ronronar, Modo::Ronronar, false));
        assert!(clicavel(Modo::Desperto, Modo::Ronronar, false));
    }

    #[test]
    fn o_limiar_da_bateria_e_dez_por_cento() {
        assert_eq!(LIMIAR_BATERIA, 10);
    }

    #[test]
    fn abaixo_do_limiar_descarregando_e_critico() {
        assert!(bateria_critica(bateria(9, true)));
        assert!(bateria_critica(bateria(0, true)));
        assert!(!bateria_critica(bateria(10, true)));
        assert!(!bateria_critica(bateria(80, true)));
    }

    #[test]
    fn na_tomada_ou_sem_bateria_nunca_e_critico() {
        assert!(!bateria_critica(bateria(3, false)));
        assert!(!bateria_critica(None));
    }

    #[test]
    fn desperto_com_bateria_critica_vira_ronronar() {
        assert_eq!(
            reconciliar(Modo::Desperto, bateria(9, true)),
            Reconciliacao { modo: Modo::Ronronar, por_bateria: true }
        );
    }

    #[test]
    fn desperto_com_bateria_boa_ou_na_tomada_segue_desperto() {
        for b in [bateria(50, true), bateria(4, false), None] {
            assert_eq!(
                reconciliar(Modo::Desperto, b),
                Reconciliacao { modo: Modo::Desperto, por_bateria: false }
            );
        }
    }

    #[test]
    fn ronronar_nunca_vira_desperto_sozinho() {
        for b in [bateria(3, true), bateria(90, false), None] {
            assert_eq!(
                reconciliar(Modo::Ronronar, b),
                Reconciliacao { modo: Modo::Ronronar, por_bateria: false }
            );
        }
    }
}
