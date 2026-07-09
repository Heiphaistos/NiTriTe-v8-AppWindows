import { invoke, invokeRaw, useNotificationStore, type Solution } from "./scanExportHelpers";
import type { ScanResult, BatteryDetailed } from "@/types/diagnostic";

export async function exportScanJson(
  scanResult: ScanResult | null,
  scanProblems: string[],
  batteries: BatteryDetailed[],
  scanSolutions: Solution[]
) {
  if (!scanResult) return;
  try {
    const { save } = await import("@tauri-apps/plugin-dialog");
    const filePath = await save({ defaultPath: "scan_total.json", filters: [{ name: "JSON", extensions: ["json"] }] });
    if (!filePath) return;
    await invoke("save_content_to_path", {
      path: filePath,
      content: JSON.stringify({ generated: new Date().toISOString(), problems: scanProblems, solutions: scanSolutions, batteries, scan: scanResult }, null, 2)
    });
    useNotificationStore().success("Scan exporté (.json)", filePath);
    await invokeRaw("open_path", { path: filePath }).catch(() => {});
  } catch (e: any) { useNotificationStore().error("Erreur export", String(e)); }
}
