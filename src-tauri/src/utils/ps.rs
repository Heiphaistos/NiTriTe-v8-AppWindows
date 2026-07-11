#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

/// Lance un script PowerShell sans fenêtre CMD visible.
///
/// Décode via `decode_output` (UTF-8 d'abord, repli codepage OEM) plutôt que
/// `from_utf8_lossy` : de nombreux scripts émettent du texte FR accentué
/// (« Activé », « non trouvée »…) que PowerShell encode en OEM (CP850 FR) faute
/// de `$OutputEncoding` — from_utf8_lossy le transformait en mojibake. Les
/// scripts déjà UTF-8/JSON restent inchangés (fast path UTF-8, aucune régression).
pub fn ps(script: &str) -> Result<String, String> {
    let out = std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .creation_flags(0x08000000)
        .output()
        .map_err(|e| e.to_string())?;
    #[cfg(target_os = "windows")]
    { Ok(crate::maintenance::commands::decode_output(&out.stdout).trim().to_string()) }
    #[cfg(not(target_os = "windows"))]
    { Ok(String::from_utf8_lossy(&out.stdout).trim().to_string()) }
}

/// Préambule PowerShell définissant la fonction `Loc-Counter` : traduit un nom
/// de compteur de performance ANGLAIS vers son libellé LOCALISÉ via l'index
/// perflib du registre.
///
/// Indispensable car `Get-Counter` n'accepte que des chemins localisés : les
/// chemins anglais codés en dur (`\GPU Engine(*)\Utilization Percentage`)
/// échouent silencieusement sur Windows non-anglophone (FR : « Moteur GPU »,
/// « Pourcentage d'utilisation »), renvoyant 0 partout. Sur Windows anglais la
/// traduction renvoie le nom d'origine (aucune régression) ; si le registre est
/// illisible on retombe aussi sur le nom d'origine.
///
/// Usage : préfixer le script avec ce préambule, puis construire le chemin avec
/// `"\{0}(*)\{1}" -f (Loc-Counter 'GPU Engine'), (Loc-Counter 'Utilization Percentage')`.
pub const LOC_COUNTER_PRELUDE: &str = r#"
$__pe = (Get-ItemProperty 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Perflib\009' -EA SilentlyContinue).Counter
$__pl = (Get-ItemProperty 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Perflib\CurrentLanguage' -EA SilentlyContinue).Counter
function Loc-Counter($n) {
    if (-not $__pe -or -not $__pl) { return $n }
    for ($i = 1; $i -lt $__pe.Count; $i += 2) {
        if ($__pe[$i] -ieq $n) {
            $id = $__pe[$i-1]
            for ($j = 0; $j -lt ($__pl.Count - 1); $j += 2) {
                if ($__pl[$j] -eq $id) { return $__pl[$j+1] }
            }
            return $n
        }
    }
    return $n
}
"#;
