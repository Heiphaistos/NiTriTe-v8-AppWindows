// Composable utilitaire pour exporter des données en CSV ou JSON
export function useExportData() {
  function exportCSV(data: Record<string, unknown>[], filename: string) {
    if (!data.length) return;
    const keys = Object.keys(data[0]);
    const header = keys.join(';');
    const rows = data.map(row =>
      keys.map(k => {
        const v = row[k] ?? '';
        const s = String(v).replace(/"/g, '""');
        return s.includes(';') || s.includes('\n') || s.includes('"') ? `"${s}"` : s;
      }).join(';')
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
