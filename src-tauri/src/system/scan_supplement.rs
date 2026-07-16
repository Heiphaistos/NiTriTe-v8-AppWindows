use serde::Serialize;
use std::process::Command;
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

// ============================================================================
// Structs exposés dans ScanResult
// ============================================================================

#[derive(Debug, Clone, Serialize, Default)]
pub struct BitlockerVolume {
    pub drive: String,
    pub protection_status: String,
    pub encryption_percent: u32,
    pub recovery_password: String,
    pub protectors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct StorageItem {
    pub model: String,
    pub size_gb: f64,
    pub media_type: String,
    pub interface_type: String,
    pub health: String,
    pub power_on_hours: u32,
    pub power_on_count: u32,
    pub rpm: u32,
}

#[derive(Debug, Default)]
pub struct ScanSupplement {
    pub windows_product_key: String,
    pub office_product_key: String,
    pub office_name: String,
    pub bitlocker_volumes: Vec<BitlockerVolume>,
    pub motherboard: String,
    pub ram_detail: String,
    pub ram_slots: Vec<String>,
    pub cpu_threads: u32,
    pub cpu_frequency_ghz: f64,
    pub cpu_socket: String,
    pub cpu_l3_mb: u32,
    pub storage_items: Vec<StorageItem>,
    pub monitors_detail: String,
}

// ============================================================================
// Collecte PowerShell unique (toutes infos supplémentaires)
// ============================================================================

pub fn collect_scan_supplement() -> ScanSupplement {
    let ps = r#"
$out = @{}

# === Clé Windows ===
function Get-WinKey {
    try {
        $sls = Get-WmiObject -Class SoftwareLicensingService -ErrorAction SilentlyContinue
        if ($sls -and $sls.OA3xOriginalProductKey) { return $sls.OA3xOriginalProductKey }
    } catch {}
    try {
        $raw = (Get-ItemProperty 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion' -ErrorAction SilentlyContinue).DigitalProductId
        if ($null -eq $raw -or $raw.Length -lt 67) { return "" }
        $offset = 52
        $isWin8 = [int][math]::Floor($raw[66] / 6) -band 1
        $raw[66] = ($raw[66] -band 0xF7) -bor (($isWin8 -band 2) * 4)
        $maps = "BCDFGHJKMPQRTVWXY2346789"; $result = ""; $n = 0
        for ($i = 24; $i -ge 0; $i--) {
            $n = 0
            for ($j = 14; $j -ge 0; $j--) {
                $n = ($n * 256) -bxor [int]$raw[$j + $offset]
                $raw[$j + $offset] = [int][math]::Floor($n / 24); $n = $n % 24
            }
            $result = $maps[$n] + $result
            if ($i % 5 -eq 0 -and $i -ne 0) { $result = "-" + $result }
        }
        return $result
    } catch { return "" }
}
$out.WinKey = Get-WinKey

# === Clé Office ===
function Get-OfficeKey {
    $maps = "BCDFGHJKMPQRTVWXY2346789"
    $isCTR = Test-Path 'HKLM:\SOFTWARE\Microsoft\Office\ClickToRun\Configuration' -ErrorAction SilentlyContinue
    $bases = @('HKLM:\SOFTWARE\Microsoft\Office\16.0\Registration','HKLM:\SOFTWARE\Microsoft\Office\15.0\Registration','HKLM:\SOFTWARE\Microsoft\Office\14.0\Registration','HKLM:\SOFTWARE\Wow6432Node\Microsoft\Office\16.0\Registration','HKLM:\SOFTWARE\Wow6432Node\Microsoft\Office\15.0\Registration','HKLM:\SOFTWARE\Wow6432Node\Microsoft\Office\14.0\Registration')
    foreach ($b in $bases) {
        if (-not (Test-Path $b -ErrorAction SilentlyContinue)) { continue }
        foreach ($sk in (Get-ChildItem $b -ErrorAction SilentlyContinue)) {
            try {
                $raw = (Get-ItemProperty $sk.PSPath -ErrorAction SilentlyContinue).DigitalProductId
                if ($null -eq $raw -or $raw.Length -lt 67) { continue }
                $rc = [byte[]]$raw.Clone()
                $offset = 52; $result = ""
                for ($i = 24; $i -ge 0; $i--) {
                    $n = 0
                    for ($j = 14; $j -ge 0; $j--) {
                        $n = ($n * 256) -bxor [int]$rc[$j + $offset]
                        $rc[$j + $offset] = [int][math]::Floor($n / 24); $n = $n % 24
                    }
                    $result = $maps[$n] + $result
                    if ($i % 5 -eq 0 -and $i -ne 0) { $result = "-" + $result }
                }
                if ($result -match '^[BCDFGHJKMPQRTVWXY2346789]{5}-[BCDFGHJKMPQRTVWXY2346789]{5}-[BCDFGHJKMPQRTVWXY2346789]{5}-[BCDFGHJKMPQRTVWXY2346789]{5}-[BCDFGHJKMPQRTVWXY2346789]{5}$') { return $result }
            } catch {}
        }
    }
    try {
        $lp = Get-WmiObject -Query "SELECT Name,PartialProductKey FROM SoftwareLicensingProduct WHERE PartialProductKey IS NOT NULL AND (Name LIKE '%Office%' OR Name LIKE '%Microsoft 365%')" -ErrorAction SilentlyContinue | Select-Object -First 1
        if ($lp -and $lp.PartialProductKey) {
            $t = if ($isCTR) { "C2R/365 — clé non stockée" } else { "MSI" }
            return "XXXXX-XXXXX-XXXXX-XXXXX-$($lp.PartialProductKey) ($t)"
        }
    } catch {}
    if ($isCTR) { return "Office 365/C2R — clé non récupérable (abonnement)" }
    return ""
}
$out.OfficeKey = Get-OfficeKey

# Nom Office
try {
    $offProd = Get-WmiObject -Query "SELECT Name,PartialProductKey FROM SoftwareLicensingProduct WHERE PartialProductKey IS NOT NULL AND (Name LIKE '*Office*' OR Name LIKE '*Microsoft 365*')" -ErrorAction SilentlyContinue | Select-Object -First 1
    $out.OfficeName = if ($offProd) { [string]$offProd.Name } else { "" }
} catch { $out.OfficeName = "" }

# === BitLocker ===
try {
    $vols = Get-BitLockerVolume -ErrorAction SilentlyContinue
    $out.BitLocker = @($vols | ForEach-Object {
        $rk = $_.KeyProtector | Where-Object { $_.KeyProtectorType -eq 'RecoveryPassword' } | Select-Object -First 1
        @{
            drive  = [string]$_.MountPoint
            status = [string]$_.ProtectionStatus
            pct    = [int]$_.EncryptionPercentage
            rk     = if ($rk -and $rk.RecoveryPassword) { [string]$rk.RecoveryPassword } else { "" }
            proto  = @($_.KeyProtector | Select-Object -ExpandProperty KeyProtectorType | ForEach-Object { [string]$_ })
        }
    })
} catch { $out.BitLocker = @() }

# === Carte mère ===
try {
    $mb = Get-WmiObject Win32_BaseBoard -ErrorAction SilentlyContinue | Select-Object -First 1
    $out.Mobo = if ($mb) { "$($mb.Manufacturer) $($mb.Product) (v$($mb.Version))" } else { "" }
} catch { $out.Mobo = "" }

# === RAM détail ===
try {
    $sticks = Get-WmiObject Win32_PhysicalMemory -ErrorAction SilentlyContinue
    $total = ($sticks | Measure-Object -Property Capacity -Sum).Sum / 1GB
    $count = ($sticks | Measure-Object).Count
    $speed = ($sticks | Select-Object -First 1).Speed
    $type = switch (($sticks | Select-Object -First 1).SMBIOSMemoryType) {
        24 { "DDR3" } 26 { "DDR4" } 34 { "DDR5" } default { "DDR" }
    }
    $out.RAM = "$count barrette(s) — $([math]::Round($total,0)) GB $type-$speed"
} catch { $out.RAM = "" }

# === CPU threads + fréquence + socket + cache L3 ===
try {
    $cpu = Get-WmiObject Win32_Processor -ErrorAction SilentlyContinue | Select-Object -First 1
    $out.CpuThreads = [int]$cpu.NumberOfLogicalProcessors
    $out.CpuGhz = [math]::Round($cpu.MaxClockSpeed / 1000.0, 2)
    $out.CpuSocket = if ($cpu.SocketDesignation) { [string]$cpu.SocketDesignation } else { "" }
    $l3kb = if ($cpu.L3CacheSize -and [long]$cpu.L3CacheSize -gt 0) { [long]$cpu.L3CacheSize } else { 0 }
    $out.CpuL3MB = [math]::Round($l3kb / 1024, 0)
} catch { $out.CpuThreads = 0; $out.CpuGhz = 0.0; $out.CpuSocket = ""; $out.CpuL3MB = 0 }

# === RAM par slot ===
try {
    $slotList = @()
    Get-WmiObject Win32_PhysicalMemory -ErrorAction SilentlyContinue | ForEach-Object {
        $t = switch ($_.SMBIOSMemoryType) { 24 { "DDR3" } 26 { "DDR4" } 34 { "DDR5" } default { "DDR" } }
        $gb = [math]::Round($_.Capacity / 1GB, 0)
        $speed = if ($_.Speed) { "-$($_.Speed)" } else { "" }
        $slot = if ($_.DeviceLocator) { $_.DeviceLocator } else { "Slot?" }
        $slotList += "$slot : $gb GB $t$speed"
    }
    $out.RamSlots = $slotList
} catch { $out.RamSlots = @() }

# === Écrans ===
try {
    $monitors = Get-WmiObject Win32_DesktopMonitor -ErrorAction SilentlyContinue | Where-Object { $_.PNPDeviceID }
    $names = @($monitors | ForEach-Object { "$($_.Name) ($($_.ScreenWidth)x$($_.ScreenHeight))" })
    $out.Monitors = if ($names.Count -gt 0) { $names -join " | " } else {
        $gpu = Get-WmiObject Win32_VideoController -ErrorAction SilentlyContinue | Select-Object -First 1
        if ($gpu) { "$($gpu.CurrentHorizontalResolution)x$($gpu.CurrentVerticalResolution) @ $($gpu.CurrentRefreshRate)Hz" } else { "" }
    }
} catch { $out.Monitors = "" }

# === Stockage physique + SMART ===
try {
    $disks = Get-PhysicalDisk -ErrorAction SilentlyContinue
    $out.Storage = @($disks | ForEach-Object {
        $disk = $_
        $rel = $null
        try { $rel = $disk | Get-StorageReliabilityCounter -ErrorAction SilentlyContinue } catch {}
        $bus = [string]$disk.BusType
        $iface = switch ($bus) { "NVMe" { "NVMe" } "SATA" { "SATA" } "SAS" { "SAS" } "USB" { "USB" } default { $bus } }
        $mt = [string]$disk.MediaType
        if ($mt -eq "Unspecified" -or $mt -eq "") {
            $n = ([string]$disk.FriendlyName).ToLower()
            $mt = if ($n -match "nvme|ssd") { "SSD" } elseif ($n -match "hdd|hard") { "HDD" } else { "Inconnu" }
        }
        @{
            model  = [string]$disk.FriendlyName
            size   = [math]::Round($disk.Size / 1e9, 0)
            type   = $mt
            iface  = $iface
            health = [string]$disk.HealthStatus
            hours  = if ($rel -and $rel.PowerOnHours) { [int]$rel.PowerOnHours } else { 0 }
            count  = if ($rel -and $rel.StartStopCycleCount) { [int]$rel.StartStopCycleCount } else { 0 }
            rpm    = if ($disk.SpindleSpeed -and [int]$disk.SpindleSpeed -gt 0) { [int]$disk.SpindleSpeed } else { 0 }
        }
    })
} catch { $out.Storage = @() }

$out | ConvertTo-Json -Depth 4 -Compress
"#;

    #[cfg(target_os = "windows")]
    {
        let output = Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", ps])
            .creation_flags(0x08000000)
            .output();

        if let Ok(o) = output {
            // decode_output : le script renvoie des libellés accentués codés en dur
            // ("clé non récupérable", "clé non stockée") sans préambule UTF8.
            let text = crate::maintenance::commands::decode_output(&o.stdout);
            let v: serde_json::Value = match serde_json::from_str(text.trim()) {
                Ok(v) => v,
                Err(_) => return ScanSupplement::default(),
            };

            let bitlocker = v["BitLocker"].as_array().map(|arr| {
                arr.iter().map(|b| BitlockerVolume {
                    drive: b["drive"].as_str().unwrap_or("").to_string(),
                    protection_status: b["status"].as_str().unwrap_or("").to_string(),
                    encryption_percent: b["pct"].as_u64().unwrap_or(0) as u32,
                    recovery_password: b["rk"].as_str().unwrap_or("").to_string(),
                    protectors: b["proto"].as_array().map(|p| {
                        p.iter().filter_map(|s| s.as_str().map(|x| x.to_string())).collect()
                    }).unwrap_or_default(),
                }).collect()
            }).unwrap_or_default();

            let storage_items = v["Storage"].as_array().map(|arr| {
                arr.iter().map(|s| StorageItem {
                    model: s["model"].as_str().unwrap_or("").to_string(),
                    size_gb: s["size"].as_f64().unwrap_or(0.0),
                    media_type: s["type"].as_str().unwrap_or("").to_string(),
                    interface_type: s["iface"].as_str().unwrap_or("").to_string(),
                    health: s["health"].as_str().unwrap_or("").to_string(),
                    power_on_hours: s["hours"].as_u64().unwrap_or(0) as u32,
                    power_on_count: s["count"].as_u64().unwrap_or(0) as u32,
                    rpm: s["rpm"].as_u64().unwrap_or(0) as u32,
                }).collect()
            }).unwrap_or_default();

            let ram_slots = v["RamSlots"].as_array().map(|a| {
                a.iter().filter_map(|s| s.as_str().map(|x| x.to_string())).collect()
            }).unwrap_or_default();

            return ScanSupplement {
                windows_product_key: v["WinKey"].as_str().unwrap_or("").to_string(),
                office_product_key: v["OfficeKey"].as_str().unwrap_or("").to_string(),
                office_name: v["OfficeName"].as_str().unwrap_or("").to_string(),
                bitlocker_volumes: bitlocker,
                motherboard: v["Mobo"].as_str().unwrap_or("").to_string(),
                ram_detail: v["RAM"].as_str().unwrap_or("").to_string(),
                ram_slots,
                cpu_threads: v["CpuThreads"].as_u64().unwrap_or(0) as u32,
                cpu_frequency_ghz: v["CpuGhz"].as_f64().unwrap_or(0.0),
                cpu_socket: v["CpuSocket"].as_str().unwrap_or("").to_string(),
                cpu_l3_mb: v["CpuL3MB"].as_u64().unwrap_or(0) as u32,
                storage_items,
                monitors_detail: v["Monitors"].as_str().unwrap_or("").to_string(),
            };
        }
    }
    ScanSupplement::default()
}
