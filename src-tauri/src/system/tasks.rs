use serde::Serialize;
use std::process::Command;
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[derive(Debug, Serialize, Clone)]
pub struct ScheduledTask {
    pub name: String,
    pub path: String,
    pub state: String,
    pub last_run_time: String,
    pub next_run_time: String,
    pub last_task_result: i64,
    pub trigger: String,
    pub author: String,
    pub description: String,
    pub run_as_user: String,
}

pub fn collect_scheduled_tasks() -> Vec<ScheduledTask> {
    #[cfg(target_os = "windows")]
    {
        let ps = r#"
try {
    $tasks = Get-ScheduledTask -ErrorAction Stop | Where-Object { $_.TaskPath -notlike "\Microsoft\Windows\*" -or $_.State -ne "Disabled" } |
        Select-Object -First 300
    $result = foreach ($t in $tasks) {
        try {
            $info = $t | Get-ScheduledTaskInfo -ErrorAction SilentlyContinue
            $trigger = if ($t.Triggers) { ($t.Triggers | Select-Object -First 1).GetType().Name -replace 'Trigger',''; } else { "Aucun" }
            [PSCustomObject]@{
                Name        = $t.TaskName
                Path        = $t.TaskPath
                State       = $t.State.ToString()
                LastRun     = if ($info -and $info.LastRunTime -and $info.LastRunTime.Year -gt 1970) { $info.LastRunTime.ToString("dd/MM/yyyy HH:mm") } else { "Jamais" }
                NextRun     = if ($info -and $info.NextRunTime -and $info.NextRunTime.Year -gt 1970) { $info.NextRunTime.ToString("dd/MM/yyyy HH:mm") } else { "N/A" }
                LastResult  = if ($info) { $info.LastTaskResult } else { 0 }
                Trigger     = $trigger
                Author      = if ($t.Principal) { $t.Principal.UserId } else { "" }
                Description = if ($t.Description) { $t.Description } else { "" }
                RunAsUser   = if ($t.Principal) { $t.Principal.UserId } else { "" }
            }
        } catch {}
    }
    $result | ConvertTo-Json -Depth 2 -Compress
} catch {
    "[]"
}
"#;
        let out = Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", ps])
            .creation_flags(0x08000000)
            .output();
        if let Ok(o) = out {
            // decode_output : Description des tâches planifiées intégrées est
            // souvent en FR accentué sur Windows FR.
            let raw = crate::maintenance::commands::decode_output(&o.stdout);
            let trimmed = raw.trim();
            if !trimmed.is_empty() && trimmed != "[]" {
                let arr: Vec<serde_json::Value> = serde_json::from_str(trimmed)
                    .unwrap_or_else(|_| serde_json::from_str(&format!("[{}]", trimmed)).unwrap_or_default());
                return arr.iter().filter_map(|v| {
                    Some(ScheduledTask {
                        name: v["Name"].as_str()?.to_string(),
                        path: v["Path"].as_str().unwrap_or("").to_string(),
                        state: v["State"].as_str().unwrap_or("Unknown").to_string(),
                        last_run_time: v["LastRun"].as_str().unwrap_or("Jamais").to_string(),
                        next_run_time: v["NextRun"].as_str().unwrap_or("N/A").to_string(),
                        last_task_result: v["LastResult"].as_i64().unwrap_or(0),
                        trigger: v["Trigger"].as_str().unwrap_or("").to_string(),
                        author: v["Author"].as_str().unwrap_or("").to_string(),
                        description: v["Description"].as_str().unwrap_or("").to_string(),
                        run_as_user: v["RunAsUser"].as_str().unwrap_or("").to_string(),
                    })
                }).collect();
            }
        }
        vec![]
    }
    #[cfg(not(target_os = "windows"))]
    vec![]
}
