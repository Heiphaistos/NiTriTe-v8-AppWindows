// Composable utilitaire pour exporter des données en CSV ou JSON

/**
 * Échappe une valeur pour une cellule CSV, en couvrant les deux risques :
 *  1. Injection de formule Excel/Sheets : une cellule débutant par = + - @ tab
 *     ou CR est exécutée comme une formule à l'ouverture (y compris entre
 *     guillemets). On la neutralise par une apostrophe, sauf nombre négatif.
 *  2. Corruption structurelle : guillemets doublés, et mise entre guillemets
 *     si la valeur contient un délimiteur (, ou ;) ou un saut de ligne.
 */
export function csvCell(value: unknown): string {
  let s = String(value ?? '');
  if (/^[=+\-@\t\r]/.test(s) && !/^-?\d+(\.\d+)?$/.test(s)) s = `'${s}`;
  s = s.replace(/"/g, '""');
  return /[",;\n\r]/.test(s) ? `"${s}"` : s;
}

export function useExportData() {
  function exportCSV(data: Record<string, unknown>[], filename: string) {
    if (!data.length) return;
    const keys = Object.keys(data[0]);
    const header = keys.map(csvCell).join(';');
    const rows = data.map(row =>
      keys.map(k => csvCell(row[k])).join(';')
    );
    const csv = [header, ...rows].join('\n');
    const blob = new Blob(['\ufeff' + csv], { type: 'text/csv;charset=utf-8' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url; a.download = filename + '.csv'; a.click();
    URL.revokeObjectURL(url);
  }

  function exportJSON(data: unknown, filename: string) {
    const json = JSON.stringify(data, null, 2);
    const blob = new Blob([json], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url; a.download = filename + '.json'; a.click();
    URL.revokeObjectURL(url);
  }

  function exportTXT(lines: string[], filename: string) {
    const txt = lines.join('\n');
    const blob = new Blob([txt], { type: 'text/plain;charset=utf-8' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url; a.download = filename + '.txt'; a.click();
    URL.revokeObjectURL(url);
  }

  return { exportCSV, exportJSON, exportTXT };
}
