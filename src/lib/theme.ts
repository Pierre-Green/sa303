// Thème clair/sombre synchronisé avec la préférence système, sans bascule
// manuelle. `.dark` (déjà défini dans style.css) est ajoutée/retirée sur
// <html> dès que l'OS change, sans attendre un rechargement.

export function syncThemeWithSystem() {
  const media = window.matchMedia("(prefers-color-scheme: dark)");

  function apply(isDark: boolean) {
    document.documentElement.classList.toggle("dark", isDark);
  }

  apply(media.matches);
  media.addEventListener("change", (e) => apply(e.matches));
}
