use serde::Serialize;
use tauri::Emitter;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[derive(Debug, Clone, Serialize)]
pub struct DepsProgress {
    pub checked: u8,
    pub total: u8,
    pub item: Dependency,
    pub done: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct Dependency {
    pub id: String,
    pub name: String,
    pub category: String,
    pub description: String,
    pub installed: bool,
    pub version: String,
    pub winget_id: String,
    pub install_url: String,
}

fn run_cmd(cmd: &str, args: &[&str]) -> (bool, String) {
    let mut command = std::process::Command::new(cmd);
    command.args(args);
    #[cfg(target_os = "windows")]
    command.creation_flags(0x08000000);
    match command.output() {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            let combined = format!("{}{}", stdout, stderr);
            let first_line = combined.lines().next().unwrap_or("").trim().to_string();
            (out.status.success() || !stdout.trim().is_empty(), first_line)
        }
        Err(_) => (false, String::new()),
    }
}

fn check(cmd: &str, version_arg: &str) -> (bool, String) {
    run_cmd(cmd, &[version_arg])
}

fn check_ps(ps_expr: &str) -> (bool, String) {
    let mut command = std::process::Command::new("powershell");
    command.args(["-NoProfile", "-NonInteractive", "-Command", ps_expr]);
    #[cfg(target_os = "windows")]
    command.creation_flags(0x08000000);
    match command.output() {
        Ok(out) => {
            let out_str = String::from_utf8_lossy(&out.stdout).trim().to_string();
            (!out_str.is_empty() && out.status.success(), out_str)
        }
        Err(_) => (false, String::new()),
    }
}

macro_rules! dep {
    ($id:expr, $name:expr, $cat:expr, $desc:expr, $ok:expr, $v:expr, $wid:expr, $url:expr) => {
        Dependency {
            id: $id.into(), name: $name.into(), category: $cat.into(),
            description: $desc.into(), installed: $ok, version: $v,
            winget_id: $wid.into(), install_url: $url.into(),
        }
    };
}

/// Toutes les vérifications s'exécutent en parallèle via thread::scope.
/// Réduit le temps de ~10-15s (séquentiel) à ~1-2s (parallèle).
pub fn check_all() -> Vec<Dependency> {
    std::thread::scope(|s| {
        // === Développement ===
        let h_git = s.spawn(|| {
            let (ok, v) = check("git", "--version");
            dep!("git","Git","Développement","Gestionnaire de versions distribué",ok,v,"Git.Git","https://git-scm.com/download/win")
        });
        let h_node = s.spawn(|| {
            let (ok, v) = check("node", "--version");
            dep!("nodejs","Node.js","Développement","Runtime JavaScript (LTS)",ok,v,"OpenJS.NodeJS.LTS","https://nodejs.org")
        });
        let h_python = s.spawn(|| {
            let (ok, v) = check("python", "--version");
            dep!("python","Python","Développement","Langage de script Python 3",ok,v,"Python.Python.3.12","https://python.org")
        });
        let h_rust = s.spawn(|| {
            let (ok, v) = check("rustc", "--version");
            dep!("rust","Rust (rustc)","Développement","Compilateur Rust via rustup",ok,v,"Rustlang.Rustup","https://rustup.rs")
        });
        let h_vscode = s.spawn(|| {
            let (ok, v) = check("code", "--version");
            let short = v.lines().next().unwrap_or("").to_string();
            dep!("vscode","Visual Studio Code","Développement","Éditeur de code Microsoft",ok,short,"Microsoft.VisualStudioCode","https://code.visualstudio.com")
        });
        let h_dotnet = s.spawn(|| {
            let (ok, v) = check("dotnet", "--version");
            dep!("dotnet",".NET Runtime","Développement","Runtime .NET Microsoft",ok,v,"Microsoft.DotNet.Runtime.8","https://dot.net")
        });

        // === Gestionnaires ===
        let h_winget = s.spawn(|| {
            let (ok, v) = check_ps("if (Get-Command winget -EA SilentlyContinue) { (winget --version) } else { exit 1 }");
            dep!("winget","WinGet","Gestionnaires","Gestionnaire de paquets Windows officiel",ok,v,"","ms-windows-store://pdp/?productid=9NBLGGH4NNS1")
        });
        let h_scoop = s.spawn(|| {
            let (ok, v) = check_ps("if (Get-Command scoop -EA SilentlyContinue) { scoop --version 2>&1 | Select-Object -First 1 } else { exit 1 }");
            dep!("scoop","Scoop","Gestionnaires","Gestionnaire de paquets sans admin",ok,v,"","https://scoop.sh")
        });
        let h_choco = s.spawn(|| {
            let (ok, v) = check_ps("if (Get-Command choco -EA SilentlyContinue) { choco --version } else { exit 1 }");
            dep!("chocolatey","Chocolatey","Gestionnaires","Gestionnaire de paquets Windows",ok,v,"","https://chocolatey.org/install")
        });

        // === Outils Système ===
        let h_curl = s.spawn(|| {
            let (ok, v) = check("curl", "--version");
            let short = v.lines().next().unwrap_or("").to_string();
            dep!("curl","curl","Outils Système","Outil de transfert HTTP/S en ligne de commande",ok,short,"curl.curl","https://curl.se")
        });
        let h_7zip = s.spawn(|| {
            let (ok, v) = check("7z", "i");
            let short = v.lines().next().unwrap_or("").to_string();
            dep!("7zip","7-Zip","Outils Système","Archiveur haute compression",ok,short,"7zip.7zip","https://7-zip.org")
        });
        let h_npp = s.spawn(|| {
            let (ok, _) = check_ps("if (Test-Path 'C:\\Program Files\\Notepad++\\notepad++.exe') { 'present' } else { exit 1 }");
            dep!("notepadpp","Notepad++","Outils Système","Éditeur de texte avancé",ok,String::new(),"Notepad++.Notepad++","https://notepad-plus-plus.org")
        });
        let h_ssh = s.spawn(|| {
            let (ok, _) = check_ps("if (Get-Command ssh -EA SilentlyContinue) { 'present' } else { exit 1 }");
            dep!("openssh","OpenSSH Client","Outils Système","Client SSH intégré Windows",ok,String::new(),"Microsoft.OpenSSH.Beta","")
        });

        // === Médias & Productivité ===
        let h_vlc = s.spawn(|| {
            let (ok, _) = check_ps("if (Test-Path 'C:\\Program Files\\VideoLAN\\VLC\\vlc.exe') { 'present' } else { exit 1 }");
            dep!("vlc","VLC Media Player","Médias","Lecteur multimédia universel",ok,String::new(),"VideoLAN.VLC","https://www.videolan.org")
        });
        let h_pt = s.spawn(|| {
            let (ok, _) = check_ps("if (Get-ItemProperty HKLM:\\Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\* | Where-Object { $_.DisplayName -like '*PowerToys*' }) { 'present' } else { exit 1 }");
            dep!("powertoys","PowerToys","Productivité","Suite d'utilitaires Microsoft pour power users",ok,String::new(),"Microsoft.PowerToys","")
        });

        // === Runtimes ===
        let h_java = s.spawn(|| {
            let (ok, v) = check_ps("if (Get-Command java -EA SilentlyContinue) { java -version 2>&1 | Select-Object -First 1 } else { exit 1 }");
            dep!("java","Java (JRE)","Runtimes","Java Runtime Environment",ok,v,"Oracle.JavaRuntimeEnvironment","https://java.com")
        });
        let h_go = s.spawn(|| {
            let (ok, v) = check_ps("if (Get-Command go -EA SilentlyContinue) { go version } else { exit 1 }");
            dep!("golang","Go","Runtimes","Langage Go (runtime + compilateur)",ok,v,"GoLang.Go","https://go.dev")
        });

        vec![
            h_git.join().unwrap_or_else(|_| dep!("git","Git","Développement","",false,String::new(),"Git.Git","")),
            h_node.join().unwrap_or_else(|_| dep!("nodejs","Node.js","Développement","",false,String::new(),"OpenJS.NodeJS.LTS","")),
            h_python.join().unwrap_or_else(|_| dep!("python","Python","Développement","",false,String::new(),"Python.Python.3.12","")),
            h_rust.join().unwrap_or_else(|_| dep!("rust","Rust","Développement","",false,String::new(),"Rustlang.Rustup","")),
            h_vscode.join().unwrap_or_else(|_| dep!("vscode","VS Code","Développement","",false,String::new(),"Microsoft.VisualStudioCode","")),
            h_dotnet.join().unwrap_or_else(|_| dep!("dotnet",".NET","Développement","",false,String::new(),"Microsoft.DotNet.Runtime.8","")),
            h_winget.join().unwrap_or_else(|_| dep!("winget","WinGet","Gestionnaires","",false,String::new(),"","")),
            h_scoop.join().unwrap_or_else(|_| dep!("scoop","Scoop","Gestionnaires","",false,String::new(),"","")),
            h_choco.join().unwrap_or_else(|_| dep!("chocolatey","Chocolatey","Gestionnaires","",false,String::new(),"","")),
            h_curl.join().unwrap_or_else(|_| dep!("curl","curl","Outils Système","",false,String::new(),"curl.curl","")),
            h_7zip.join().unwrap_or_else(|_| dep!("7zip","7-Zip","Outils Système","",false,String::new(),"7zip.7zip","")),
            h_npp.join().unwrap_or_else(|_| dep!("notepadpp","Notepad++","Outils Système","",false,String::new(),"Notepad++.Notepad++","")),
            h_ssh.join().unwrap_or_else(|_| dep!("openssh","OpenSSH","Outils Système","",false,String::new(),"Microsoft.OpenSSH.Beta","")),
            h_vlc.join().unwrap_or_else(|_| dep!("vlc","VLC","Médias","",false,String::new(),"VideoLAN.VLC","")),
            h_pt.join().unwrap_or_else(|_| dep!("powertoys","PowerToys","Productivité","",false,String::new(),"Microsoft.PowerToys","")),
            h_java.join().unwrap_or_else(|_| dep!("java","Java","Runtimes","",false,String::new(),"Oracle.JavaRuntimeEnvironment","")),
            h_go.join().unwrap_or_else(|_| dep!("golang","Go","Runtimes","",false,String::new(),"GoLang.Go","")),
        ]
    })
}

/// Vérifie chaque dépendance et émet un événement `deps:progress` au fur et à mesure.
/// Permet à l'UI d'afficher les résultats en temps réel sans freeze.
pub fn check_all_streaming(app: &tauri::AppHandle) {
    // Closures pour chaque check — exécutées en parallèle
    type CheckFn = Box<dyn Fn() -> Dependency + Send + Sync>;
    let checks: Vec<CheckFn> = vec![
        Box::new(|| { let (ok, v) = check("git", "--version"); dep!("git","Git","Développement","Gestionnaire de versions distribué",ok,v,"Git.Git","https://git-scm.com/download/win") }),
        Box::new(|| { let (ok, v) = check("node", "--version"); dep!("nodejs","Node.js","Développement","Runtime JavaScript (LTS)",ok,v,"OpenJS.NodeJS.LTS","https://nodejs.org") }),
        Box::new(|| { let (ok, v) = check("python", "--version"); dep!("python","Python","Développement","Langage de script Python 3",ok,v,"Python.Python.3.12","https://python.org") }),
        Box::new(|| { let (ok, v) = check("rustc", "--version"); dep!("rust","Rust (rustc)","Développement","Compilateur Rust via rustup",ok,v,"Rustlang.Rustup","https://rustup.rs") }),
        Box::new(|| { let (ok, v) = check("code", "--version"); let s=v.lines().next().unwrap_or("").to_string(); dep!("vscode","Visual Studio Code","Développement","Éditeur de code Microsoft",ok,s,"Microsoft.VisualStudioCode","https://code.visualstudio.com") }),
        Box::new(|| { let (ok, v) = check("dotnet", "--version"); dep!("dotnet",".NET Runtime","Développement","Runtime .NET Microsoft",ok,v,"Microsoft.DotNet.Runtime.8","https://dot.net") }),
        Box::new(|| { let (ok, v) = check_ps("if (Get-Command winget -EA SilentlyContinue) { (winget --version) } else { exit 1 }"); dep!("winget","WinGet","Gestionnaires","Gestionnaire de paquets Windows officiel",ok,v,"","ms-windows-store://pdp/?productid=9NBLGGH4NNS1") }),
        Box::new(|| { let (ok, v) = check_ps("if (Get-Command scoop -EA SilentlyContinue) { scoop --version 2>&1 | Select-Object -First 1 } else { exit 1 }"); dep!("scoop","Scoop","Gestionnaires","Gestionnaire de paquets sans admin",ok,v,"","https://scoop.sh") }),
        Box::new(|| { let (ok, v) = check_ps("if (Get-Command choco -EA SilentlyContinue) { choco --version } else { exit 1 }"); dep!("chocolatey","Chocolatey","Gestionnaires","Gestionnaire de paquets Windows",ok,v,"","https://chocolatey.org/install") }),
        Box::new(|| { let (ok, v) = check("curl", "--version"); let s=v.lines().next().unwrap_or("").to_string(); dep!("curl","curl","Outils Système","Outil de transfert HTTP/S en ligne de commande",ok,s,"curl.curl","https://curl.se") }),
        Box::new(|| { let (ok, v) = check("7z", "i"); let s=v.lines().next().unwrap_or("").to_string(); dep!("7zip","7-Zip","Outils Système","Archiveur haute compression",ok,s,"7zip.7zip","https://7-zip.org") }),
        Box::new(|| { let (ok, _) = check_ps("if (Test-Path 'C:\\Program Files\\Notepad++\\notepad++.exe') { 'present' } else { exit 1 }"); dep!("notepadpp","Notepad++","Outils Système","Éditeur de texte avancé",ok,String::new(),"Notepad++.Notepad++","https://notepad-plus-plus.org") }),
        Box::new(|| { let (ok, _) = check_ps("if (Get-Command ssh -EA SilentlyContinue) { 'present' } else { exit 1 }"); dep!("openssh","OpenSSH Client","Outils Système","Client SSH intégré Windows",ok,String::new(),"Microsoft.OpenSSH.Beta","") }),
        Box::new(|| { let (ok, _) = check_ps("if (Test-Path 'C:\\Program Files\\VideoLAN\\VLC\\vlc.exe') { 'present' } else { exit 1 }"); dep!("vlc","VLC Media Player","Médias","Lecteur multimédia universel",ok,String::new(),"VideoLAN.VLC","https://www.videolan.org") }),
        Box::new(|| { let (ok, _) = check_ps("if (Get-ItemProperty HKLM:\\Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\* | Where-Object { $_.DisplayName -like '*PowerToys*' }) { 'present' } else { exit 1 }"); dep!("powertoys","PowerToys","Productivité","Suite d'utilitaires Microsoft pour power users",ok,String::new(),"Microsoft.PowerToys","") }),
        Box::new(|| { let (ok, v) = check_ps("if (Get-Command java -EA SilentlyContinue) { java -version 2>&1 | Select-Object -First 1 } else { exit 1 }"); dep!("java","Java (JRE)","Runtimes","Java Runtime Environment",ok,v,"Oracle.JavaRuntimeEnvironment","https://java.com") }),
        Box::new(|| { let (ok, v) = check_ps("if (Get-Command go -EA SilentlyContinue) { go version } else { exit 1 }"); dep!("golang","Go","Runtimes","Langage Go (runtime + compilateur)",ok,v,"GoLang.Go","https://go.dev") }),
    ];

    let total = checks.len() as u8;
    // Exécuter en parallèle mais émettre les résultats dès qu'ils arrivent
    let (tx, rx) = std::sync::mpsc::channel::<(u8, Dependency)>();

    std::thread::scope(|s| {
        for (i, check_fn) in checks.iter().enumerate() {
            let tx = tx.clone();
            s.spawn(move || {
                let dep = check_fn();
                let _ = tx.send((i as u8, dep));
            });
        }
        drop(tx); // ferme le sender scope
        let mut received = 0u8;
        let mut ordered: Vec<Option<Dependency>> = vec![None; total as usize];
        for (idx, dep) in rx {
            ordered[idx as usize] = Some(dep);
            received += 1;
            // Émettre chaque résultat immédiatement
            if let Some(ref item) = ordered[idx as usize] {
                let _ = app.emit("deps:progress", DepsProgress {
                    checked: received,
                    total,
                    item: item.clone(),
                    done: received == total,
                });
            }
        }
    });
}

pub fn install_via_winget(winget_id: &str) -> Result<String, String> {
    if winget_id.is_empty() {
        return Err("Pas d'ID winget — installez manuellement.".into());
    }
    let mut cmd = std::process::Command::new("winget");
    cmd.args(["install", "--id", winget_id, "--accept-source-agreements", "--accept-package-agreements", "-h"]);
    #[cfg(target_os = "windows")]
    cmd.creation_flags(0x08000000);
    match cmd.output() {
        Ok(out) => {
            // decode_output : winget émet des messages d'erreur localisés FR
            // accentués ("package introuvable"...), sans $OutputEncoding préalable.
            let stdout = crate::maintenance::commands::decode_output(&out.stdout);
            let stderr = crate::maintenance::commands::decode_output(&out.stderr);
            if out.status.success() {
                Ok(format!("✓ {} installé avec succès", winget_id))
            } else {
                Err(format!("Erreur winget: {}{}", stdout, stderr))
            }
        }
        Err(e) => Err(format!("Impossible de lancer winget: {}", e)),
    }
}
