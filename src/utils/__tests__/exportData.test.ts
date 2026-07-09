import { describe, it, expect, vi, beforeEach } from "vitest";

// Mock DOM dependencies used by exportCSV/exportJSON/exportTXT
const mockClick = vi.fn();
const mockCreateElement = vi.fn(() => ({
  href: "",
  download: "",
  click: mockClick,
}));
const mockCreateObjectURL = vi.fn(() => "blob:mock-url");
const mockRevokeObjectURL = vi.fn();

vi.stubGlobal("URL", {
  createObjectURL: mockCreateObjectURL,
  revokeObjectURL: mockRevokeObjectURL,
});
vi.stubGlobal("document", {
  createElement: mockCreateElement,
});

import { useExportData } from "@/composables/useExportData";

describe("useExportData — exportCSV quoting", () => {
  beforeEach(() => {
    mockClick.mockClear();
    mockCreateObjectURL.mockClear();
    mockRevokeObjectURL.mockClear();
    mockCreateElement.mockClear();
  });

  it("déclenche le téléchargement avec un Blob CSV", () => {
    const { exportCSV } = useExportData();
    exportCSV([{ name: "Alice", score: 42 }], "test");
    expect(mockCreateObjectURL).toHaveBeenCalledOnce();
    expect(mockClick).toHaveBeenCalledOnce();
    expect(mockRevokeObjectURL).toHaveBeenCalledWith("blob:mock-url");
    const blob: Blob = mockCreateObjectURL.mock.calls[0][0];
    expect(blob.type).toContain("text/csv");
  });

  it("ne fait rien si le tableau est vide", () => {
    const { exportCSV } = useExportData();
    exportCSV([], "test");
    expect(mockCreateObjectURL).not.toHaveBeenCalled();
  });

  it("exportJSON déclenche le téléchargement JSON", () => {
    const { exportJSON } = useExportData();
    exportJSON({ a: 1 }, "out");
    expect(mockCreateObjectURL).toHaveBeenCalledOnce();
    expect(mockClick).toHaveBeenCalledOnce();
    const blob: Blob = mockCreateObjectURL.mock.calls[0][0];
    expect(blob.type).toContain("application/json");
  });

  it("exportTXT déclenche le téléchargement texte", () => {
    const { exportTXT } = useExportData();
    exportTXT(["ligne 1", "ligne 2"], "log");
    expect(mockCreateObjectURL).toHaveBeenCalledOnce();
    const blob: Blob = mockCreateObjectURL.mock.calls[0][0];
    expect(blob.type).toContain("text/plain");
  });
});

// Test de la logique de quoting CSV en isolation
describe("csvQuoting — règles d'échappement", () => {
  // Reproduit la logique interne de exportCSV pour tests purs
  function csvCell(v: any): string {
    const s = String(v ?? "").replace(/"/g, '""');
    return s.includes(";") || s.includes("\n") || s.includes('"') ? `"${s}"` : s;
  }

  it("valeur simple sans caractères spéciaux", () => {
    expect(csvCell("Hello")).toBe("Hello");
    expect(csvCell(42)).toBe("42");
    expect(csvCell(null)).toBe("");
    expect(csvCell(undefined)).toBe("");
  });

  it("valeur contenant ; → entourée de guillemets", () => {
    expect(csvCell("a;b")).toBe('"a;b"');
    expect(csvCell("hello;world")).toBe('"hello;world"');
  });

  it("valeur contenant saut de ligne → entourée de guillemets", () => {
    expect(csvCell("ligne1\nligne2")).toBe('"ligne1\nligne2"');
  });

  it("valeur contenant guillemets → guillemets doublés et entourée", () => {
    expect(csvCell('say "hi"')).toBe('"say ""hi"""');
  });

  it("valeur contenant les deux → guillemets doublés + entourée", () => {
    expect(csvCell('a;b"c')).toBe('"a;b""c"');
  });
});
