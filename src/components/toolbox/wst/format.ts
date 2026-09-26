// Formatage des résultats WST (conventions d'affichage de la spec). Aucun
// calcul : `null` s'affiche « — ».

type Value = number | null | undefined;

export const khz = (hz: Value) => (hz == null ? "—" : `${(hz / 1000).toFixed(1)} kHz`);
export const deg = (value: Value) => (value == null ? "—" : `${value.toFixed(1)}°`);
export const mm = (value: Value) => (value == null ? "—" : `${value.toFixed(1)} mm`);
export const metres = (value: Value) => (value == null ? "—" : `${value.toFixed(1)} m`);
export const db = (value: Value) => (value == null ? "—" : `${value.toFixed(1)} dB`);
export const ratio = (value: Value) => (value == null ? "—" : value.toFixed(3));
