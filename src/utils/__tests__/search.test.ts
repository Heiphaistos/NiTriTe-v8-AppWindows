import { describe, it, expect } from "vitest";
import { ref } from "vue";
import { useSearch } from "@/composables/useSearch";

interface App {
  name: string;
  publisher: string | null;
  version: string;
}

const APPS: App[] = [
  { name: "Google Chrome", publisher: "Google LLC", version: "125.0" },
  { name: "VLC Media Player", publisher: "VideoLAN", version: "3.0.21" },
  { name: "7-Zip", publisher: null, version: "23.01" },
  { name: "Visual Studio Code", publisher: "Microsoft", version: "1.89" },
];

describe("useSearch — filtrage réactif", () => {
  it("retourne tous les items si query vide", () => {
    const items = ref(APPS);
    const { filtered } = useSearch(items, ["name", "publisher"]);
    expect(filtered.value).toHaveLength(4);
  });

  it("filtre par nom (insensible à la casse)", () => {
    const items = ref(APPS);
    const { query, filtered } = useSearch(items, ["name", "publisher"]);
    query.value = "chrome";
    expect(filtered.value).toHaveLength(1);
    expect(filtered.value[0].name).toBe("Google Chrome");
  });

  it("filtre par publisher", () => {
    const items = ref(APPS);
    const { query, filtered } = useSearch(items, ["name", "publisher"]);
    query.value = "microsoft";
    expect(filtered.value).toHaveLength(1);
    expect(filtered.value[0].name).toBe("Visual Studio Code");
  });

  it("ignore les champs null sans crash", () => {
    const items = ref(APPS);
    const { query, filtered } = useSearch(items, ["name", "publisher"]);
    query.value = "7-zip";
    // 7-Zip a publisher null — ne doit pas crasher
    expect(filtered.value).toHaveLength(1);
    expect(filtered.value[0].name).toBe("7-Zip");
  });

  it("retourne [] si aucun résultat", () => {
    const items = ref(APPS);
    const { query, filtered, hasResults } = useSearch(items, ["name", "publisher"]);
    query.value = "zzz_inexistant";
    expect(filtered.value).toHaveLength(0);
    expect(hasResults.value).toBe(false);
  });

  it("hasResults est true quand des résultats existent", () => {
    const items = ref(APPS);
    const { query, hasResults } = useSearch(items, ["name", "publisher"]);
    query.value = "vlc";
    expect(hasResults.value).toBe(true);
  });

  it("clear() remet query à vide et restaure tous les items", () => {
    const items = ref(APPS);
    const { query, filtered, clear } = useSearch(items, ["name", "publisher"]);
    query.value = "google";
    expect(filtered.value).toHaveLength(1);
    clear();
    expect(query.value).toBe("");
    expect(filtered.value).toHaveLength(4);
  });

  it("réactif : mise à jour si items change", () => {
    const items = ref<App[]>([...APPS]);
    const { query, filtered } = useSearch(items, ["name"]);
    query.value = "Firefox";
    expect(filtered.value).toHaveLength(0);
    items.value.push({ name: "Mozilla Firefox", publisher: "Mozilla", version: "125" });
    expect(filtered.value).toHaveLength(1);
  });

  it("ignore les espaces en début/fin de query", () => {
    const items = ref(APPS);
    const { query, filtered } = useSearch(items, ["name"]);
    query.value = "  vlc  ";
    expect(filtered.value).toHaveLength(1);
  });
});
