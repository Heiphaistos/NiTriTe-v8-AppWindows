import { describe, it, expect } from "vitest";
import { kbStr, fullRegPath } from "@/composables/scan/scanExportHelpers";

describe("kbStr", () => {
  it("valeur < 1024 → affiche KB", () => {
    expect(kbStr(512)).toBe("512 KB");
    expect(kbStr(0)).toBe("0 KB");
    expect(kbStr(1023)).toBe("1023 KB");
  });

  it("valeur >= 1024 → affiche MB (arrondi)", () => {
    expect(kbStr(1024)).toBe("1 MB");
    expect(kbStr(2048)).toBe("2 MB");
    expect(kbStr(1500)).toBe("1 MB");
    expect(kbStr(1536)).toBe("2 MB");
  });

  it("grandes valeurs converties en MB", () => {
    expect(kbStr(10240)).toBe("10 MB");
  });
});

describe("fullRegPath", () => {
  it("expanse HKCU avec sous-chemin", () => {
    expect(fullRegPath("HKCU\\Software\\Microsoft")).toBe("HKEY_CURRENT_USER\\Software\\Microsoft");
  });

  it("expanse HKCU seul", () => {
    expect(fullRegPath("HKCU")).toBe("HKEY_CURRENT_USER");
  });

  it("expanse HKLM avec sous-chemin", () => {
    expect(fullRegPath("HKLM\\SYSTEM\\CurrentControlSet")).toBe("HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet");
  });

  it("expanse HKCR", () => {
    expect(fullRegPath("HKCR\\.exe")).toBe("HKEY_CLASSES_ROOT\\.exe");
  });

  it("expanse HKU", () => {
    expect(fullRegPath("HKU\\S-1-5-21")).toBe("HKEY_USERS\\S-1-5-21");
  });

  it("ajoute le nom si fourni (chemin sans barre oblique finale)", () => {
    expect(fullRegPath("HKCU\\Software", "Run")).toBe("HKEY_CURRENT_USER\\Software\\Run");
  });

  it("n'ajoute pas de double backslash si chemin se termine par \\", () => {
    expect(fullRegPath("HKCU\\Software\\", "Run")).toBe("HKEY_CURRENT_USER\\Software\\Run");
  });

  it("chemin non-HKCU/HKLM est retourné tel quel", () => {
    expect(fullRegPath("UNKNOWN\\Path")).toBe("UNKNOWN\\Path");
  });

  it("sans nom — retourne seulement le chemin expansé", () => {
    expect(fullRegPath("HKLM\\SOFTWARE")).toBe("HKEY_LOCAL_MACHINE\\SOFTWARE");
  });
});
