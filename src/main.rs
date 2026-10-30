mod aviso;
mod bateria;
mod estado;
mod modelo;
mod tema;
mod trava;
mod vigia;

use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use cosmic::app::Core;
use cosmic::applet::menu_button;
use cosmic::iced::{Alignment, Color, Length, Limits, Subscription, window, window::Id};
use cosmic::surface::action::{app_popup, destroy_popup};
use cosmic::widget::{Column, Row, icon, svg, text};
use cosmic::{Action, Element, Task, theme};

use estado::Estado;
use modelo::Modo;
use tema::Tema;
use trava::{Segurada, Sistema};

const ID: &str = "io.github.meowsystem.Soneca";

const ARTE_DESPERTO: &[u8] = include_bytes!("../data/icones/soneca-desperto.svg");
const ARTE_RONRONAR: &[u8] = include_bytes!("../data/icones/soneca-ronronar.svg");

const ICONE_ROTULO: u16 = 16;
const PASSO_DESPERTO: Duration = Duration::from_secs(30);

const SYS: &str = "/sys";

const ERRO_TRAVA: &str = "O sistema recusou o Desperto";
const ERRO_GRAVAR: &str = "O modo não foi gravado";

fn arte_do_modo(m: Modo) -> &'static [u8] {
    match m {
        Modo::Desperto => ARTE_DESPERTO,
        Modo::Ronronar => ARTE_RONRONAR,
    }
}

fn cor_do_acento(t: &cosmic::Theme) -> Color {
    t.cosmic().accent.base.into()
}

fn svg_no_acento() -> theme::Svg {
    theme::Svg::Custom(Rc::new(|t| svg::Style {
        color: Some(cor_do_acento(t)),
    }))
}

fn texto_no_acento() -> theme::Text {
    theme::Text::Custom(|t| cosmic::iced::widget::text::Style {
        color: Some(cor_do_acento(t)),
        ..Default::default()
    })
}

struct Applet {
    core: Core,
    popup: Option<Id>,
    modo: Modo,
    estado: Option<Estado>,
    logind: Sistema,
    trava: Option<Arc<Segurada>>,
    travando: bool,
    erro: Option<String>,
    tema: Tema,
}

#[derive(Clone, Debug)]
enum Message {
    AbreFecha,
    PopupFechou(Id),
    Tique,
    Mudou,
    Escolher(Modo),
    Travou(Result<Arc<Segurada>, String>),
    Avisou,
    Surface(cosmic::surface::Action),
}

impl Applet {
    fn mostrado(&self) -> Modo {
        modelo::mostrado(self.modo, self.trava.is_some())
    }

    fn sincronizar(&mut self) -> Task<Action<Message>> {
        match self.modo {
            Modo::Ronronar => {
                self.trava = None;
                Task::none()
            }
            Modo::Desperto if self.trava.is_some() || self.travando => Task::none(),
            Modo::Desperto => {
                self.travando = true;
                let logind = self.logind.clone();
                cosmic::task::future(async move {
                    Message::Travou(trava::segurar(&logind).await.map(Arc::new))
                })
            }
        }
    }

    fn conferir(&mut self) -> Task<Action<Message>> {
        let no_disco = self.estado.as_ref().map_or(self.modo, Estado::ler);
        let r = modelo::reconciliar(no_disco, bateria::ler(Path::new(SYS)));
        self.modo = r.modo;
        let mut tarefas = Vec::new();
        if r.por_bateria {
            if let Some(e) = &self.estado
                && let Err(falha) = e.gravar(Modo::Ronronar)
            {
                eprintln!("meowsystem-soneca: {}: {falha}", e.arquivo().display());
                self.erro = Some(ERRO_GRAVAR.into());
            }
            tarefas.push(cosmic::task::future(async {
                let _ = aviso::avisar("Soneca", &aviso::texto_da_bateria()).await;
                Message::Avisou
            }));
        }
        tarefas.push(self.sincronizar());
        Task::batch(tarefas)
    }

    fn bloco_modos(&self) -> Element<'_, Message> {
        let mut col = Column::new();
        let mostrado = self.mostrado();
        for m in Modo::TODOS {
            let marcado = mostrado == m;
            let arte = icon::icon(icon::from_svg_bytes(arte_do_modo(m)).symbolic(true)).size(ICONE_ROTULO);
            let nome = text::body(m.nome()).width(Length::Fill);
            let mut linha = Row::new().spacing(8).align_y(Alignment::Center);
            if marcado {
                linha = linha
                    .push(arte.class(svg_no_acento()))
                    .push(nome.class(texto_no_acento()))
                    .push(
                        icon::from_name("object-select-symbolic")
                            .size(ICONE_ROTULO)
                            .icon()
                            .class(svg_no_acento()),
                    );
            } else {
                linha = linha.push(arte).push(nome);
            }
            col = col.push(menu_button(linha).on_press_maybe(
                modelo::clicavel(m, self.modo, self.trava.is_some()).then_some(Message::Escolher(m)),
            ));
        }
        col.into()
    }
}

impl cosmic::Application for Applet {
    type Executor = cosmic::SingleThreadExecutor;
    type Flags = ();
    type Message = Message;
    const APP_ID: &'static str = ID;

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<Action<Self::Message>>) {
        let estado = Estado::da_casa(ID);
        let modo = estado.as_ref().map_or(Modo::PADRAO, Estado::ler);
        let mut applet = Self {
            core,
            popup: None,
            modo,
            estado,
            logind: Sistema::default(),
            trava: None,
            travando: false,
            erro: None,
            tema: Tema::load(ID),
        };
        let cores = applet.aplicar_tema(cosmic::theme::active().cosmic().is_dark, false);
        let conferir = applet.conferir();
        (applet, Task::batch([cores, conferir]))
    }

    fn on_close_requested(&self, id: window::Id) -> Option<Message> {
        Some(Message::PopupFechou(id))
    }

    fn update(&mut self, message: Message) -> Task<Action<Self::Message>> {
        match message {
            Message::AbreFecha => return self.abre_fecha(),
            Message::PopupFechou(id) => {
                if self.popup == Some(id) {
                    self.popup = None;
                }
            }
            Message::Tique | Message::Mudou => return self.conferir(),
            Message::Escolher(m) => {
                if let Some(e) = &self.estado
                    && let Err(falha) = e.gravar(m)
                {
                    eprintln!("meowsystem-soneca: {}: {falha}", e.arquivo().display());
                    self.erro = Some(ERRO_GRAVAR.into());
                    return Task::none();
                }
                self.erro = None;
                self.modo = m;
                return self.conferir();
            }
            Message::Travou(r) => {
                self.travando = false;
                match r {
                    Ok(t) if self.modo == Modo::Desperto && self.trava.is_none() => {
                        self.trava = Some(t);
                        self.erro = None;
                    }
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("meowsystem-soneca: o logind recusou a trava: {e}");
                        if self.modo == Modo::Desperto {
                            self.erro = Some(ERRO_TRAVA.into());
                        }
                    }
                }
            }
            Message::Avisou => {}
            Message::Surface(a) => return tarefa_de_superficie(a),
        }
        Task::none()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        let mudancas = match &self.estado {
            Some(e) => vigia::mudancas(e.arquivo().to_path_buf()).map(|()| Message::Mudou),
            None => Subscription::none(),
        };
        let tique = match self.modo {
            Modo::Desperto => cosmic::iced::time::every(PASSO_DESPERTO).map(|_| Message::Tique),
            Modo::Ronronar => Subscription::none(),
        };
        Subscription::batch([mudancas, tique])
    }

    fn view(&self) -> Element<'_, Message> {
        let botao = self
            .core
            .applet
            .icon_button_from_handle(icon::from_svg_bytes(arte_do_modo(self.mostrado())).symbolic(true))
            .on_press_down(Message::AbreFecha);
        self.core
            .applet
            .applet_tooltip(botao, self.mostrado().nome(), self.popup.is_some(), Message::Surface, None)
            .into()
    }

    fn view_window(&self, _id: Id) -> Element<'_, Message> {
        let mut conteudo: Column<'_, Message, cosmic::Theme> = Column::new().push(self.bloco_modos());
        if let Some(e) = &self.erro {
            conteudo = conteudo.push(cosmic::applet::padded_control(text::caption(e.clone())));
        }
        self.core.applet.popup_container(conteudo.padding([8, 0])).into()
    }

    fn style(&self) -> Option<cosmic::iced::theme::Style> {
        Some(cosmic::applet::style())
    }

    fn system_theme_update(
        &mut self,
        _keys: &[&'static str],
        novo: &cosmic::cosmic_theme::Theme,
    ) -> Task<Action<Self::Message>> {
        self.aplicar_tema(novo.is_dark, false)
    }

    fn system_theme_mode_update(
        &mut self,
        _keys: &[&'static str],
        modo: &cosmic::cosmic_theme::ThemeMode,
    ) -> Task<Action<Self::Message>> {
        self.aplicar_tema(modo.is_dark, false)
    }
}

impl Applet {
    fn aplicar_tema(&self, escuro: bool, antes: bool) -> Task<Action<Message>> {
        match self.tema.cosmic(escuro) {
            Some(t) => cosmic::command::set_theme(t),
            None if antes => cosmic::command::set_theme(self.core.system_theme().clone()),
            None => Task::none(),
        }
    }

    fn reler_tema(&mut self) -> Task<Action<Message>> {
        let novo = Tema::load(ID);
        if novo == self.tema {
            return Task::none();
        }
        let antes = self.tema.cosmic(true).is_some();
        self.tema = novo;
        self.aplicar_tema(cosmic::theme::active().cosmic().is_dark, antes)
    }

    fn abre_fecha(&mut self) -> Task<Action<Message>> {
        if let Some(popup) = self.popup.take() {
            return tarefa_de_superficie(destroy_popup(popup));
        }
        let Some(pai) = self.core.main_window_id() else {
            eprintln!("meowsystem-soneca: sem janela principal — popup não abre");
            return Task::none();
        };
        let abrir = tarefa_de_superficie(app_popup::<Applet>(
            |_| Default::default(),
            move |estado: &mut Applet| {
                let popup = Id::unique();
                let mut ajustes = estado.core.applet.get_popup_settings(pai, popup, None, None, None);
                ajustes.positioner.size_limits = Limits::NONE
                    .max_width(320.0)
                    .min_width(240.0)
                    .min_height(80.0)
                    .max_height(600.0);
                estado.popup = Some(popup);
                ajustes
            },
            None,
        ));
        let cores = self.reler_tema();
        Task::batch([cores, abrir, self.conferir()])
    }
}

fn tarefa_de_superficie(acao: cosmic::surface::Action) -> Task<Action<Message>> {
    cosmic::task::message(cosmic::Action::Cosmic(cosmic::app::Action::Surface(acao)))
}

enum Pedido {
    Estado,
    Gravar(Modo),
}

fn pedido_da_linha(args: impl IntoIterator<Item = String>) -> Option<Pedido> {
    args.into_iter().find_map(|a| match a.as_str() {
        "--estado" => Some(Pedido::Estado),
        "--desperto" => Some(Pedido::Gravar(Modo::Desperto)),
        "--ronronar" => Some(Pedido::Gravar(Modo::Ronronar)),
        _ => None,
    })
}

fn atender(pedido: Pedido) -> i32 {
    let Some(estado) = Estado::da_casa(ID) else {
        eprintln!("meowsystem-soneca: sem HOME nem XDG_STATE_HOME");
        return 1;
    };
    match pedido {
        Pedido::Estado => {
            println!("{}", estado.ler().chave());
            0
        }
        Pedido::Gravar(modo) => match estado.gravar(modo) {
            Ok(()) => 0,
            Err(e) => {
                eprintln!("meowsystem-soneca: {}: {e}", estado.arquivo().display());
                1
            }
        },
    }
}

fn main() -> cosmic::iced::Result {
    if let Some(pedido) = pedido_da_linha(std::env::args().skip(1)) {
        std::process::exit(atender(pedido));
    }
    cosmic::applet::run::<Applet>(())
}

#[cfg(test)]
mod testes {
    use super::*;

    fn linha(args: &[&str]) -> Option<Pedido> {
        pedido_da_linha(args.iter().map(|a| a.to_string()))
    }

    #[test]
    fn a_linha_de_comando_conhece_tres_pedidos() {
        assert!(matches!(linha(&["--estado"]), Some(Pedido::Estado)));
        assert!(matches!(linha(&["--desperto"]), Some(Pedido::Gravar(Modo::Desperto))));
        assert!(matches!(linha(&["--ronronar"]), Some(Pedido::Gravar(Modo::Ronronar))));
        assert!(linha(&[]).is_none());
        assert!(linha(&["--outra"]).is_none());
    }
}
