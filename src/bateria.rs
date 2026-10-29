use std::path::Path;

use crate::modelo::Bateria;

fn campo(pasta: &Path, nome: &str) -> String {
    std::fs::read_to_string(pasta.join(nome)).unwrap_or_default().trim().to_string()
}

pub fn ler(sys: &Path) -> Option<Bateria> {
    let mut pastas: Vec<_> = std::fs::read_dir(sys.join("class/power_supply")).ok()?.flatten().map(|e| e.path()).collect();
    pastas.sort();
    pastas
        .iter()
        .filter(|p| campo(p, "type") == "Battery" && campo(p, "scope") != "Device")
        .filter_map(|p| {
            let percentual = campo(p, "capacity").parse::<u8>().ok().filter(|n| *n <= 100)?;
            Some(Bateria { percentual, descarregando: campo(p, "status") == "Discharging" })
        })
        .min_by_key(|b| b.percentual)
}

#[cfg(test)]
mod testes {
    use super::*;
    use std::path::PathBuf;

    fn sys(nome: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("soneca-sys-{nome}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(d.join("class/power_supply")).unwrap();
        d
    }

    fn fonte(sys: &Path, nome: &str, campos: &[(&str, &str)]) {
        let p = sys.join("class/power_supply").join(nome);
        std::fs::create_dir_all(&p).unwrap();
        for (k, v) in campos {
            std::fs::write(p.join(k), format!("{v}\n")).unwrap();
        }
    }

    #[test]
    fn le_a_bateria_do_notebook() {
        let s = sys("notebook");
        fonte(&s, "BAT1", &[("type", "Battery"), ("capacity", "8"), ("status", "Discharging")]);
        fonte(&s, "ADP1", &[("type", "Mains"), ("online", "0")]);
        assert_eq!(ler(&s), Some(Bateria { percentual: 8, descarregando: true }));
        let _ = std::fs::remove_dir_all(&s);
    }

    #[test]
    fn carregando_nao_e_descarregando() {
        let s = sys("tomada");
        fonte(&s, "BAT1", &[("type", "Battery"), ("capacity", "5"), ("status", "Charging")]);
        assert_eq!(ler(&s), Some(Bateria { percentual: 5, descarregando: false }));
        fonte(&s, "BAT1", &[("type", "Battery"), ("capacity", "5"), ("status", "Not charging")]);
        assert_eq!(ler(&s), Some(Bateria { percentual: 5, descarregando: false }));
        let _ = std::fs::remove_dir_all(&s);
    }

    #[test]
    fn periferico_nao_conta() {
        let s = sys("mouse");
        fonte(&s, "hidpp_battery_0", &[("type", "Battery"), ("scope", "Device"), ("capacity", "2"), ("status", "Discharging")]);
        assert_eq!(ler(&s), None);
        let _ = std::fs::remove_dir_all(&s);
    }

    #[test]
    fn desktop_sem_bateria_ou_sem_sysfs() {
        let s = sys("desktop");
        fonte(&s, "AC", &[("type", "Mains"), ("online", "1")]);
        assert_eq!(ler(&s), None);
        let _ = std::fs::remove_dir_all(&s);
        assert_eq!(ler(&s), None);
    }

    #[test]
    fn leitura_ruim_e_ignorada() {
        let s = sys("ruim");
        fonte(&s, "BAT0", &[("type", "Battery"), ("capacity", "cem"), ("status", "Discharging")]);
        fonte(&s, "BAT1", &[("type", "Battery"), ("capacity", "250"), ("status", "Discharging")]);
        assert_eq!(ler(&s), None);
        let _ = std::fs::remove_dir_all(&s);
    }

    #[test]
    fn com_duas_baterias_vale_a_mais_baixa() {
        let s = sys("duas");
        fonte(&s, "BAT0", &[("type", "Battery"), ("capacity", "60"), ("status", "Discharging")]);
        fonte(&s, "BAT1", &[("type", "Battery"), ("capacity", "7"), ("status", "Discharging")]);
        assert_eq!(ler(&s), Some(Bateria { percentual: 7, descarregando: true }));
        let _ = std::fs::remove_dir_all(&s);
    }
}
