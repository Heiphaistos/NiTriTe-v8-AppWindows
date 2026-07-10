export { exportScanTxt } from "./scan/useScanExportTxt";
export { exportScanHtml } from "./scan/useScanExportHtml";
export { exportScanMd } from "./scan/useScanExportMd";
export { exportScanJson } from "./scan/useScanExportJson";
export type { Solution } from "./scan/scanExportHelpers";

import { exportScanTxt } from "./scan/useScanExportTxt";
import { exportScanHtml } from "./scan/useScanExportHtml";
import { exportScanMd } from "./scan/useScanExportMd";
import { exportScanJson } from "./scan/useScanExportJson";

export function useScanExport() {
  return { exportScanTxt, exportScanHtml, exportScanMd, exportScanJson };
}
