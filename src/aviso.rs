use std::collections::HashMap;

use crate::modelo::LIMIAR_BATERIA;

pub fn texto_da_bateria() -> String {
    format!("Bateria abaixo de {LIMIAR_BATERIA}%: o PC volta a dormir como o sistema manda.")
}

pub async fn avisar(titulo: &str, corpo: &str) -> Result<(), String> {
    let conexao = zbus::Connection::session().await.map_err(|e| e.to_string())?;
    let proxy = zbus::Proxy::new(
        &conexao,
        "org.freedesktop.Notifications",
        "/org/freedesktop/Notifications",
        "org.freedesktop.Notifications",
    )
    .await
    .map_err(|e| e.to_string())?;
    let acoes: Vec<&str> = Vec::new();
    let dicas: HashMap<&str, zbus::zvariant::Value<'_>> = HashMap::new();
    proxy
        .call::<_, _, u32>(
            "Notify",
            &("Soneca", 0u32, "io.github.meowsystem.Soneca-symbolic", titulo, corpo, acoes, dicas, 8000i32),
        )
        .await
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn o_texto_cita_o_limiar_do_codigo() {
        assert!(texto_da_bateria().contains("10%"));
    }
}
