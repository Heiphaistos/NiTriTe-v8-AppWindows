export interface NavItem {
  id: string;
  label: string;
  icon: string;
  route: string;
  badge?: number;
}

export interface NavSection {
  title: string;
  items: NavItem[];
}

export const navigationSections: NavSection[] = [
  {
    title: "Systeme",
    items: [
      { id: "dashboard", label: "Tableau de bord", icon: "layout-dashboard", route: "/" },
      { id: "diagnostic", label: "Diagnostic", icon: "stethoscope", route: "/diagnostic" },
      { id: "optimizations", label: "Optimisations", icon: "zap", route: "/optimizations" },
    ],
  },
  {
    title: "Logiciels",
    items: [
      { id: "tools", label: "Outils Systeme", icon: "wrench", route: "/tools" },
      { id: "master-install", label: "Master Install", icon: "download", route: "/master-install" },
      { id: "portables", label: "Apps Portables", icon: "package", route: "/portables" },
      { id: "os-downloads", label: "OS & USB Tools", icon: "hard-drive", route: "/os-downloads" },
    ],
  },
  {
    title: "Performance",
    items: [
      { id: "temperatures",  label: "Températures",        icon: "thermometer",  route: "/temperatures" },
      { id: "benchmark",     label: "Benchmark",           icon: "gauge",        route: "/benchmark" },
      { id: "perf-history",  label: "Historique Perf.",    icon: "bar-chart-3",  route: "/perf-history" },
    ],
  },
  {
    title: "Avancé (BETA)",
    items: [
      { id: "clone",           label: "Clonage Système",     icon: "copy",            route: "/clone" },
      { id: "data-recovery",   label: "Récupération Données",icon: "database",        route: "/data-recovery" },
      { id: "disk-visualizer", label: "Visualiseur Disque",  icon: "pie-chart",       route: "/disk-visualizer" },
      { id: "duplicate-finder",label: "Doublons",            icon: "files",           route: "/duplicate-finder" },
      { id: "big-files",       label: "Gros Fichiers",       icon: "file-search",     route: "/big-files" },
      { id: "hash-checker",    label: "Hash Checker",        icon: "hash",            route: "/hash-checker" },
      { id: "boot-manager",    label: "Boot Manager",        icon: "server",          route: "/boot-manager" },
      { id: "hosts-editor",    label: "Éditeur Hosts",       icon: "globe",           route: "/hosts-editor" },
      { id: "bsod-analyzer",   label: "Analyse BSOD",        icon: "bug",             route: "/bsod-analyzer" },
      { id: "wsl",             label: "WSL Linux",           icon: "terminal-square", route: "/wsl" },
      { id: "restore-points",  label: "Restauration",        icon: "shield-check",    route: "/restore-points" },
      { id: "docker",          label: "Docker Manager",      icon: "container",       route: "/docker" },
    ],
  },
  {
    title: "Maintenance",
    items: [
      { id: "updates", label: "Mises a jour", icon: "refresh-cw", route: "/updates" },
      { id: "drivers", label: "Drivers", icon: "cpu", route: "/drivers" },
      { id: "uninstaller", label: "Désinstallateur", icon: "trash-2", route: "/uninstaller" },
      { id: "cleaner", label: "Nettoyeur Avancé", icon: "sparkles", route: "/cleaner" },
      { id: "backup", label: "Sauvegarde", icon: "save", route: "/backup" },
      { id: "scanvirus", label: "Scan Antivirus", icon: "shield", route: "/scanvirus" },
      { id: "dependencies", label: "Dépendances", icon: "package", route: "/dependencies" },
    ],
  },
  {
    title: "Reseau & Terminal",
    items: [
      { id: "network",       label: "Reseau",             icon: "wifi",          route: "/network" },
      { id: "dns-switcher",  label: "DNS Switcher",        icon: "globe",         route: "/dns-switcher" },
      { id: "wifi-analyzer", label: "WiFi Analyzer",       icon: "radio",         route: "/wifi-analyzer" },
      { id: "port-scanner",  label: "Scanner de Ports",    icon: "network",       route: "/port-scanner" },
      { id: "bluetooth",     label: "Bluetooth",           icon: "bluetooth",     route: "/bluetooth" },
      { id: "terminal",      label: "Terminal",            icon: "terminal",      route: "/terminal" },
      { id: "scripts",       label: "Scripts & Snippets",  icon: "file-code",     route: "/scripts" },
    ],
  },
  {
    title: "Intelligence",
    items: [
      { id: "ai-agents", label: "Agent IA", icon: "bot", route: "/ai-agents" },
      { id: "knowledge-base", label: "Base de Connaissances", icon: "book-open", route: "/knowledge-base" },
      { id: "documentation", label: "Documentation", icon: "file-text", route: "/documentation" },
    ],
  },
  {
    title: "Rapports",
    items: [
      { id: "logs", label: "Logs", icon: "scroll-text", route: "/logs" },
      { id: "theme-editor", label: "Éditeur de Thème", icon: "palette", route: "/theme-editor" },
    ],
  },
  {
    title: "Configuration",
    items: [
      { id: "settings", label: "Paramètres", icon: "settings", route: "/settings" },
      { id: "profiles", label: "Profils", icon: "user", route: "/profiles" },
    ],
  },
  {
    title: "WinPE",
    items: [
      { id: "winpe", label: "Mode WinPE", icon: "hard-drive", route: "/winpe" },
    ],
  },
];
