use serde::Serialize;
use crate::utils::ps::ps;

#[derive(Serialize, Clone)]
pub struct DllEntry {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub company: String,
    pub description: String,
    pub version: String,
    pub location: String,  // "System32" | "SysWOW64" | "ProgramFiles"
    pub category: String,  // "Système" | "Tiers" | "Pilote"
}

fn dll_category(company: &str, location: &str) -> String {
    let c = company.to_lowercase();
    if c.contains("microsoft") || c.contains("windows") {
        "Système".to_string()
    } else if location == "System32" || location == "SysWOW64" {
        // Une DLL non-Microsoft dans System32/SysWOW64 est ajoutée par un pilote/app tierce
        "Tiers (System32)".to_string()
    } else {
        "Application".to_string()
    }
}

fn scan_dlls_sync() -> Vec<DllEntry> {
    // On scanne System32 + SysWOW64 pour les DLLs tierces (non-Microsoft)
    // + ProgramFiles pour les DLLs racines d'applications
    let script = r#"
$results = @()
$sys = [System.Environment]::SystemDirectory
$sys32 = $sys
$sys64 = if (Test-Path 'C:\Windows\SysWOW64') { 'C:\Windows\SysWOW64' } else { $null }

function Scan-Dir($dir, $loc) {
    Get-ChildItem "$dir\*.dll" -ErrorAction SilentlyContinue | ForEach-Object {
        try {
            $vi = [System.Diagnostics.FileVersionInfo]::GetVersionInfo($_.FullName)
            $company = if ($vi.CompanyName) { $vi.CompanyName.Trim() } else { '' }
            $results += [PSCustomObject]@{
                name        = $_.Name
                path        = $_.FullName
                size        = $_.Length
                company     = $company
                description = if ($vi.FileDescription) { $vi.FileDescription.Trim() } else { '' }
                version     = if ($vi.FileVersion) { $vi.FileVersion.Trim() } else { '' }
                location    = $loc
            }
        } catch {}
    }
}

Scan-Dir $sys32 'System32'
if ($sys64) { Scan-Dir $sys64 'SysWOW64' }

# DLLs dans ProgramFiles (racine seulement, pas récursif — trop lent)
$pfPaths = @('C:\Program Files','C:\Program Files (x86)')
foreach ($pf in $pfPaths) {
    if (-not (Test-Path $pf)) { continue }
    Get-ChildItem $pf -ErrorAction SilentlyContinue | ForEach-Object {
        Get-ChildItem "$($_.FullName)\*.dll" -ErrorAction SilentlyContinue | Select-Object -First 30 | ForEach-Object {
            try {
                $vi = [System.Diagnostics.FileVersionInfo]::GetVersionInfo($_.FullName)
                $company = if ($vi.CompanyName) { $vi.CompanyName.Trim() } else { '' }
                $results += [PSCustomObject]@{
                    name        = $_.Name
                    path        = $_.FullName
                    size        = $_.Length
                    company     = $company
                    description = if ($vi.FileDescription) { $vi.FileDescription.Trim() } else { '' }
                    version     = if ($vi.FileVersion) { $vi.FileVersion.Trim() } else { '' }
                    location    = 'ProgramFiles'
                }
            } catch {}
        }
    }
}

@($results) | ConvertTo-Json -Compress -Depth 2
"#;

    let out = ps(script).unwrap_or_default();
    if out.trim().is_empty() || out.trim() == "null" {
        return vec![];
    }
    let arr: Vec<serde_json::Value> = match serde_json::from_str(out.trim()) {
        Ok(v) => v,
        Err(_) => return vec![],
    };

    arr.iter().filter_map(|v| {
        let name     = v["name"].as_str().unwrap_or("").to_string();
        let path     = v["path"].as_str().unwrap_or("").to_string();
        let company  = v["company"].as_str().unwrap_or("").to_string();
        let location = v["location"].as_str().unwrap_or("").to_string();
        if name.is_empty() || path.is_empty() { return None; }
        let category = dll_category(&company, &location);
        Some(DllEntry {
            name,
            path,
            size:        v["size"].as_u64().unwrap_or(0),
            company,
            description: v["description"].as_str().unwrap_or("").to_string(),
            version:     v["version"].as_str().unwrap_or("").to_string(),
            location,
            category,
        })
    }).collect()
}

#[tauri::command]
pub async fn scan_dlls() -> Result<Vec<DllEntry>, String> {
    tokio::task::spawn_blocking(scan_dlls_sync)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_dll(path: String) -> Result<(), String> {
    // Sécurité : autoriser uniquement les .dll dans System32/SysWOW64/Program Files
    let p = std::path::Path::new(&path);
    let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    if ext != "dll" {
        return Err("Suppression refusée : extension non autorisée.".to_string());
    }
    let canonical = p.canonicalize().map_err(|e| e.to_string())?;
    let allowed = [
        r"C:\Windows\System32",
        r"C:\Windows\SysWOW64",
        r"C:\Program Files",
        r"C:\Program Files (x86)",
    ];
    let in_allowed = allowed.iter().any(|base| {
        canonical.starts_with(std::path::Path::new(base))
    });
    if !in_allowed {
        return Err("Suppression refusée : chemin hors des répertoires autorisés.".to_string());
    }
    tokio::task::spawn_blocking(move || {
        std::fs::remove_file(&canonical)
            .map_err(|e| format!("Impossible de supprimer : {}", e))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn microsoft_company_is_system() {
        assert_eq!(dll_category("Microsoft Corporation", "System32"), "Système");
        assert_eq!(dll_category("Microsoft Windows", "SysWOW64"), "Système");
    }

    #[test]
    fn windows_in_company_is_system() {
        assert_eq!(dll_category("Windows Components", "anywhere"), "Système");
    }

    #[test]
    fn third_party_in_system32_is_tiers() {
        assert_eq!(dll_category("NVidia Corporation", "System32"), "Tiers (System32)");
        assert_eq!(dll_category("Intel Inc", "SysWOW64"), "Tiers (System32)");
    }

    #[test]
    fn third_party_elsewhere_is_application() {
        assert_eq!(dll_category("Adobe Systems", "ProgramFiles"), "Application");
        assert_eq!(dll_category("Unknown Vendor", "SomeOtherLocation"), "Application");
    }

    #[test]
    fn empty_company_in_system32_is_tiers() {
        assert_eq!(dll_category("", "System32"), "Tiers (System32)");
    }

    #[test]
    fn empty_company_elsewhere_is_application() {
        assert_eq!(dll_category("", "ProgramFiles"), "Application");
    }
}
